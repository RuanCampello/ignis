use std::path::Path;

use ignis::vm::{self, VmError};

type Result<T> = std::result::Result<T, VmError>;

#[test]
fn initialise_vm() -> Result<()> {
    let class = Path::new("./sources/Person.class");
    let result = vm::run(class);
    assert!(result.is_ok());

    Ok(())
}

