//! The JVM does not rely on runtime layout of classes, interface or instances.
//! Instead, instructions refer to symbolic information provided by the `constant_pool` table.
//!
//! Specification for the runtime [constant pool] in the JVM.
//!
//! [constant pool]: https://docs.oracle.com/javase/specs/jvms/se8/html/jvms-2.html#jvms-2.5.5

use core::fmt::{Display, Formatter};
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
#[repr(u8)]
pub(crate) enum ConstantPoolEntry<'c> {
    Utf8(&'c str) = 1,
    Integer(i32) = 3,
    Float(f32) = 4,
    Long(i64) = 5,
    Double(f64) = 6,

    Class(u16) = 7,
    StringRef(u16) = 8,

    FieldRef(u16, u16) = 9,
    MethodRef(u16, u16) = 10,
    InterfaceMethodRef(u16, u16) = 11,
    NameAndType(u16, u16) = 12,
}

#[derive(Error, Debug, PartialEq)]
pub(crate) enum ConstantPoolError {
    #[error("Invalid index location: {0}")]
    InvalidIndex(u16),
    #[error("Accessed reserved slot: {0}")]
    UnusableSlot(u16),
    #[error(transparent)]
    Formatter(#[from] core::fmt::Error),
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

    fn format(&self, index: u16, f: &mut Formatter) -> Result<(), ConstantPoolError> {
        let entry = self.get(index)?;

        match entry {
            ConstantPoolEntry::Utf8(string) => f.write_str(string),
            ConstantPoolEntry::Integer(int) => write!(f, "{int}"),
            ConstantPoolEntry::Float(float) => write!(f, "{float}"),
            ConstantPoolEntry::Long(long) => write!(f, "{long}"),
            ConstantPoolEntry::Double(double) => write!(f, "{double}"),

            ConstantPoolEntry::Class(idx) | ConstantPoolEntry::StringRef(idx) => {
                return self.format(*idx, f);
            }

            ConstantPoolEntry::FieldRef(idx, info)
            | ConstantPoolEntry::MethodRef(idx, info)
            | ConstantPoolEntry::InterfaceMethodRef(idx, info)
            | ConstantPoolEntry::NameAndType(idx, info) => {
                self.format(*idx, f)?;
                write!(f, ".")?;
                Ok(self.format(*info, f)?)
            }
        }
        .map_err(Into::into)
    }

    fn format_entry(&self, index: u16, f: &mut Formatter) -> Result<(), ConstantPoolError> {
        fn format_pair(
            this: &ConstantPool,
            name: &str,
            first: u16,
            second: u16,
            f: &mut Formatter,
        ) -> Result<(), ConstantPoolError> {
            write!(f, "{name}: {} => (", first)?;
            this.format_entry(second, f)?;
            write!(f, ")").map_err(Into::into)
        }

        match self.get(index)? {
            ConstantPoolEntry::Utf8(s) => write!(f, "Utf8: \"{s}\""),
            ConstantPoolEntry::Integer(int) => write!(f, "Integer: {int}"),
            ConstantPoolEntry::Float(float) => write!(f, "Float: {float}"),
            ConstantPoolEntry::Long(int) => write!(f, "Long: {int}"),
            ConstantPoolEntry::Double(float) => write!(f, "Double: {float}"),

            ConstantPoolEntry::Class(idx) => {
                write!(f, "Class: {} => (", idx)?;
                self.format_entry(*idx, f)?;
                write!(f, ")")
            }
            ConstantPoolEntry::StringRef(idx) => {
                write!(f, "StringRef: {} => (", idx)?;
                self.format_entry(*idx, f)?;
                write!(f, ")")
            }

            ConstantPoolEntry::FieldRef(idx, info) => {
                return format_pair(self, "FieldRef", *idx, *info, f);
            }
            ConstantPoolEntry::MethodRef(idx, info) => {
                return format_pair(self, "MethodRef", *idx, *info, f);
            }
            ConstantPoolEntry::NameAndType(idx, info) => {
                return format_pair(self, "NameAndType", *idx, *info, f);
            }
            ConstantPoolEntry::InterfaceMethodRef(idx, info) => {
                return format_pair(self, "InterfaceMethodRef", *idx, *info, f);
            }
        }
        .map_err(Into::into)
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

impl<'c> Display for ConstantPool<'c> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Constant pool with size: {}", self.entries.len())?;

        for idx in 0..self.entries.len() as u16 {
            writeln!(f, "   {idx}, ")?;
            self.format_entry(idx, f).map_err(|_| std::fmt::Error)?;
        }

        Ok(())
    }
}
