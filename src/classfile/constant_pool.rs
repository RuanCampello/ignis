//! The JVM does not rely on runtime layout of classes, interface or instances.
//! Instead, instructions refer to symbolic information provided by the `constant_pool` table.
//!
//! Specification for the runtime [constant pool] in the JVM.
//!
//! [constant pool]: https://docs.oracle.com/javase/specs/jvms/se8/html/jvms-2.html#jvms-2.5.5

use std::fmt::Display;
use thiserror::Error;

/// Constant pool of a given Java class.
#[derive(Debug, Default, PartialEq, Clone)]
pub(crate) struct ConstantPool<'c> {
    entries: Vec<Option<ConstantPoolEntry<'c>>>,
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
    FieldRef(u16, u16),
    MethodRef(u16, u16),
    InterfaceMethodRef(u16, u16),
    NameAndType(u16, u16),
}

#[derive(Error, Debug, PartialEq)]
pub(crate) enum ConstantPoolError {
    #[error("Invalid index location: {0}")]
    InvalidIndex(u16),
    #[error("Accessed reserved slot: {0}")]
    UnusableSlot(u16),
}

impl<'c> ConstantPool<'c> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, entry: ConstantPoolEntry<'c>) {
        self.entries.push(Some(entry));

        if entry.uses_two_slots() {
            self.entries.push(None);
        }
    }

    /// Tries to access a [pool entry](ConstantPoolEntry) in a given index.
    ///
    /// **Note**: it uses 1-index based.
    pub fn get(&self, index: u16) -> Result<&ConstantPoolEntry, ConstantPoolError> {
        if index == 0 || index as usize > self.entries.len() {
            return Err(ConstantPoolError::InvalidIndex(index));
        }

        let idx = (index - 1) as usize;
        match self.entries.get(idx) {
            Some(Some(entry)) => Ok(entry),
            Some(None) => Err(ConstantPoolError::UnusableSlot(index)),
            None => Err(ConstantPoolError::InvalidIndex(index)),
        }
    }
}

impl<'c> ConstantPoolEntry<'c> {
    /// JVM mandates that `Long` and `Double` constraints must occupy two slots in the constant
    /// pool.
    /// This is due to historical architectural constraints and alignment, tied to the JVM's
    /// original 32-bit design and its operand stack.
    fn uses_two_slots(&self) -> bool {
        matches!(self, Self::Long(_) | Self::Double(_))
    }
}

impl<'c> Display for ConstantPoolEntry<'c> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConstantPoolEntry::Utf8(s) => write!(f, "Utf8 \"{s}\""),
            ConstantPoolEntry::Integer(n) => write!(f, "Integer {n}"),
            ConstantPoolEntry::Float(n) => write!(f, "Float {n}"),
            ConstantPoolEntry::Long(int) => write!(f, "Long {int}"),
            ConstantPoolEntry::Double(float) => write!(f, "Double {float}"),

            ConstantPoolEntry::Class(idx) => write!(f, "Class index {idx}"),

            ConstantPoolEntry::FieldRef(idx, info) => write!(f, "FieldRef: {idx}, {info}"),
            ConstantPoolEntry::MethodRef(idx, info) => write!(f, "MethodRef: {idx}, {info}"),
            ConstantPoolEntry::NameAndType(idx, info) => write!(f, "NameAndType: {idx}, {info}"),
            ConstantPoolEntry::InterfaceMethodRef(idx, info) => {
                write!(f, "InterfaceMethodRef: {idx}, {info}")
            }
        }
    }
}
