use crate::{
    Args,
    vm::{Result, runtime::RuntimeError},
};
use dashmap::mapref::entry;
use once_cell::sync::OnceCell;
use std::{
    env, fs,
    path::{Path, PathBuf},
};

static CLASSPATH: OnceCell<String> = OnceCell::new();

impl<'a> Args<'a> {
    pub(crate) fn resolve_class_path(&self) -> Result<()> {
        let mut class_path = String::from(".");

        if let Ok(cp) = env::var("CLASSPATH") {
            class_path = cp;
        }

        if let Some(cp) = self.class_path {
            class_path = cp.to_string();
        }

        if self.jar_mode {
            class_path = self.entry.to_string();
        }

        class_path = wildcards(&class_path)?;

        CLASSPATH
            .set(class_path)
            .map_err(|existing| panic!("CLASSPATH was already set: {existing}"));

        Ok(())
    }
}

fn wildcards(class_path: &str) -> Result<String> {
    if !class_path.contains('*') {
        return Ok(class_path.to_string());
    }

    let mut wildcards = Vec::new();

    for entry in class_path.split(os::path_separator()) {
        match is_wildcard(entry) {
            true => match expand(entry)? {
                Some(mut jars) => wildcards.append(&mut jars),
                _ => wildcards.push(entry.to_string()),
            },
            false => wildcards.push(entry.to_string()),
        }
    }

    Ok(wildcards.join(os::path_separator()))
}

fn expand(entry: &str) -> Result<Option<Vec<String>>> {
    let dir = match entry == "*" {
        true => PathBuf::from("."),
        _ => PathBuf::from(&entry[..entry.len() - 1]),
    };

    let Some(read_dir) = fs::read_dir(&dir).ok() else {
        return Ok(None);
    };

    let jars: Vec<_> = read_dir
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| {
            path.extension()
                .map(|ext| ext.eq_ignore_ascii_case("jar"))
                .unwrap_or(false)
        })
        .map(|path| path.to_string_lossy().into_owned())
        .collect();

    Ok((!jars.is_empty()).then(|| jars))
}

fn is_wildcard(entry: &str) -> bool {
    if !entry.ends_with('*') {
        return false;
    }

    if entry.len() > 1 {
        let bytes = entry.as_bytes();
        let previous = bytes[bytes.len() - 2];

        if previous != b'/' && previous != b'\\' {
            return false;
        }
    }

    !Path::new(entry).exists()
}

mod os {
    #[inline]
    pub const fn path_separator<'s>() -> &'s str {
        match cfg!(target_os = "windows") {
            true => ";",
            false => ":",
        }
    }
}
