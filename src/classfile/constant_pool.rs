//! The JVM does not rely on runtime layout of classes, interface or instances.
//! Instead, instructions refer to symbolic information provided by the `constant_pool` table.
//!
//! Specification for the runtime [constant pool] in the JVM.
//!
//! [constant pool]: https://docs.oracle.com/javase/specs/jvms/se8/html/jvms-2.html#jvms-2.5.5

use thiserror::Error;

/// Constant pool of a given Java class.
#[derive(Debug, Default, PartialEq, Clone)]
pub(crate) struct ConstantPool<'c> {
    entries: Vec<ConstantPoolEntry<'c>>,
}

/// A given entry in the constant pool.
///
/// It's defined by the [specification] by having a `tag` and `info`.
///
/// [specification]: https://docs.oracle.com/javase/specs/jvms/se8/html/jvms-4.html#jvms-4.4
#[derive(Debug, PartialEq, Clone, Copy)]
pub(crate) enum ConstantPoolEntry<'c> {
    Utf8(&'c str),
    Integer(i32),
    Float(f32),
    Long(i64),
    Double(f64),

    Class(u16),
    FieldRef(u16),
    MethodRef(u16, u16),
    InterfaceMethodRef(u16, u16),
    NameAndType(u16, u16),
}

#[derive(Error, Debug, PartialEq)]
pub(crate) enum ConstantPoolError {
    #[error("Invalid index location: {0}")]
    InvalidIndex(usize),
}
