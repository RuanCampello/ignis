use ignis::classfile::{Classfile, ClassfileError, FieldFlags, MethodFlags};
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
    let methods = classfile.methods_signatures(&arena)?;
    assert_eq!(
        methods,
        bumpalo::vec![
            in &arena;
            // this basically means that the function
            // takes as arguments a String and an integer (the I), and returns a void (the V)
            ("<init>", "(Ljava/lang/String;I)V"),
            // in this case, we take no arguments (see the empty parems?) and return a String
            ("getName", "()Ljava/lang/String;")
        ]
    );

    Ok(())
}

#[test]
fn employee_class() -> Result<()> {
    let arena = bumpalo::Bump::new();
    let buffer = fs::read("./tests/sources/Employee.class")?;
    let classfile = Classfile::new(&buffer, &arena)?;

    assert_eq!(classfile.version(), (68, 0)); // this file was compiled with javac 24.0.2
    assert!(classfile.is_public());
    assert_eq!(classfile.class_name(), Some("example/Employee")); // package prefix
    assert_eq!(classfile.super_class(), Some("java/lang/Object"));

    let interfaces = classfile.interface_names(&arena)?;
    assert_eq!(
        interfaces,
        bumpalo::vec![in &arena; "java/io/Serializable"].as_slice()
    );

    let methods = classfile.methods_signatures(&arena)?;
    assert_eq!(
        methods,
        bumpalo::vec![in &arena;
        ("<init>", "(Ljava/lang/String;I)V"),
        ("getSalary", "()D"),
        ("getName", "()Ljava/lang/String;"),
        ("getCompany", "()Ljava/lang/String;")],
    );

    assert!(classfile.methods[3].contains(MethodFlags::STATIC));
    assert!(classfile.fields[2].contains(FieldFlags::STATIC));
    assert!(classfile.methods[1].contains(MethodFlags::ABSTRACT));

    Ok(())
}
