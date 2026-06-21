use indexmap::IndexMap;

pub mod classfile;
mod image;
pub mod vm;

#[derive(Default)]
pub struct Args<'a> {
    pub entry: &'a str,
    pub program_args: Vec<&'a str>,
    pub system_properties: IndexMap<&'a str, &'a str>,
    pub class_path: Option<&'a str>,
    pub jar_mode: bool,
}

impl<'a> Args<'a> {
    pub fn with_entry(entry: &'a str) -> Self {
        Self {
            entry,
            ..Default::default()
        }
    }
}
