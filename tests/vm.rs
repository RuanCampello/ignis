use std::path::Path;

use ignis::vm::{self, Args, VmError};

type Result<T> = std::result::Result<T, VmError>;

#[test]
fn initialise_vm() -> Result<()> {
    let class = Path::new("./sources/Sum.class");
    let args = Args { entry: "Main" };

    let result = vm::run(args, class);
    assert!(result.is_ok());

    Ok(())
}

