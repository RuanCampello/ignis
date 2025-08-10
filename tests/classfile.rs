use ignis::classfile::{Classfile, ClassfileError};
use std::fs::{self};

type Result<T> = std::result::Result<T, ClassfileError>;

#[test]
fn person_class() -> Result<()> {
    let arena = bumpalo::Bump::new();
    let buffer = fs::read("./tests/sources/Person.class")?;
    let classfile = Classfile::new(&buffer, &arena)?;

    assert_eq!(classfile.version(), (68, 0)); // this file was compiled with javac 24.0.2
    assert!(classfile.is_public());
    assert_eq!(classfile.class_name(), Some("Person"));
    assert_eq!(classfile.super_class(), Some("java/lang/Object")); // all java's object inherit this object super class

    let fields = classfile.field_names(&arena)?;
    assert_eq!(fields, bumpalo::vec![in &arena; "name", "age"]);

    Ok(())
}
