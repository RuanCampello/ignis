use ignis::classfile::{Classfile, ClassfileError};
use std::fs::{self};

type Result<T> = std::result::Result<T, ClassfileError>;

#[test]
fn person_class() -> Result<()> {
    let arena = bumpalo::Bump::new();
    let buffer = fs::read("./tests/sources/Person.class")?;
    let classfile = Classfile::new(&buffer, &arena)?;

    Ok(())
}
