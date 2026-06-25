//! End-to-end VM scenarios

use ignis::{
    Args,
    vm::{self, VmError},
};
use rusty_fork::rusty_fork_test;
use std::path::{Path, PathBuf};

const SOURCES: &str = "tests/sources";

/// resolves a JDK home that contains a `lib/modules` jimage
fn java_home() -> Option<PathBuf> {
    let has_modules = |home: &Path| home.join("lib").join("modules").is_file();

    if let Ok(home) = std::env::var("JAVA_HOME") {
        let home = PathBuf::from(home);
        if has_modules(&home) {
            return Some(home);
        }
    }

    let preferred = [
        "/usr/lib/jvm/java-11-openjdk",
        "/usr/lib/jvm/java-17-openjdk",
        "/usr/lib/jvm/java-21-openjdk",
        "/usr/lib/jvm/java-24-jdk",
        "/usr/lib/jvm/default",
    ];
    for candidate in preferred {
        let path = PathBuf::from(candidate);
        if has_modules(&path) {
            return Some(path);
        }
    }

    std::fs::read_dir("/usr/lib/jvm")
        .ok()?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .find(|path| has_modules(path))
}

fn run(entry: &'static str) -> Result<(), VmError> {
    let home = java_home().expect("no JDK with lib/modules found, set JAVA_HOME to a JDK");

    let mut args = Args::with_entry(entry);
    args.class_path = Some(SOURCES);

    vm::run(args, home)
}

fn skip_if_no_jdk() -> bool {
    if java_home().is_none() {
        eprintln!("skipping: no JDK with lib/modules found (set JAVA_HOME)");
        return true;
    }
    false
}

rusty_fork_test! {
    #![rusty_fork(timeout_ms = 60_000)]

    #[test]
    fn runs_integer_for_loop() {
        if skip_if_no_jdk() {
            return;
        }
        let result = run("Sum");
        assert!(result.is_ok(), "Sum should run cleanly, got: {result:#?}");
    }

    #[test]
    fn prints_hello_world() {
        if skip_if_no_jdk() {
            return;
        }
        let result = run("HelloWorld");
        assert!(result.is_ok(), "HelloWorld should run cleanly, got: {result:#?}");
    }

    #[test]
    fn computes_mixed_arithmetic() {
        if skip_if_no_jdk() {
            return;
        }
        let result = run("Arithmetic");
        assert!(result.is_ok(), "Arithmetic should run cleanly, got: {result:#?}");
    }

    #[test]
    fn catches_array_index_out_of_bounds() {
        if skip_if_no_jdk() {
            return;
        }
        let result = run("Exceptions");
        assert!(result.is_ok(), "Exceptions should run cleanly, got: {result:#?}");
    }

    #[test]
    fn iterates_enum_values() {
        if skip_if_no_jdk() {
            return;
        }
        let result = run("TaskStatus");
        assert!(result.is_ok(), "TaskStatus should run cleanly, got: {result:#?}");
    }
}
