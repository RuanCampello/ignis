use ignis::Args;
use ignis::vm::{self, VmError};
use std::path::Path;

type Result<T> = std::result::Result<T, VmError>;

#[test]
fn initialise_vm() -> Result<()> {
    let class = Path::new("./sources/Sum.class");
    let args = Args::with_entry("Main");

    let result = vm::run(args, class);
    assert!(result.is_ok());

    Ok(())
}
