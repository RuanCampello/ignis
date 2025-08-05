//! The JVM does not rely on runtime layout of classes, interface or instances.
//! Instead, instructions refer to symbolic information provided by the `constant_pool` table.
//!
//! Specification for the runtime [constant pool] in the JVM.
//!
//! [constant pool]: https://docs.oracle.com/javase/specs/jvms/se8/html/jvms-2.html#jvms-2.5.5

use core::fmt::{Display, Formatter};
use std::io::{Cursor, Read, Seek, SeekFrom};
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
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            entries: Vec::with_capacity(capacity),
        }
    }

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

impl<'c> TryFrom<&mut Cursor<&'c [u8]>> for ConstantPool<'c> {
    type Error = crate::classfile::ClassfileError;

    fn try_from(reader: &mut Cursor<&'c [u8]>) -> Result<Self, Self::Error> {
        use super::ConstantPoolEntry as Entry;
        use crate::classfile::read;

        let count = {
            let mut buff = [0u8, 2];
            reader.read_exact(&mut buff)?;
            u16::from_be_bytes(buff) as usize
        };

        let mut pool = ConstantPool::with_capacity(count);

        for mut idx in (0..count) {
            let tag = read::<u8>(&[0u8], reader)?;

            let entry = match tag {
                1 => {
                    // FIXME: maybe we should support cesu8 strings, but it would be a pain in the
                    // ass to do that without allocating a new string instance or change the api to
                    // have a Cow
                    let length = read::<u16>(&[0u8; 2], reader)? as usize;
                    let pos = reader.position() as usize;
                    reader.seek(SeekFrom::Current(length as i64))?;

                    let data = reader.get_ref();
                    let bytes = &data[pos..pos + length];

                    Entry::Utf8(std::str::from_utf8(bytes)?)
                }
                3 => Entry::Integer(read::<i32>(&[0u8; 4], reader)?),
                4 => Entry::Float(read::<f32>(&[0u8; 4], reader)?),
                5 => {
                    idx += 1;
                    Entry::Long(read::<i64>(&[0u8; 8], reader)?)
                }
                6 => {
                    idx += 1;
                    Entry::Double(read::<f64>(&[0u8; 8], reader)?)
                }
                7 => Entry::Class(read::<u16>(&[0u8; 2], reader)?),
                8 => Entry::StringRef(read::<u16>(&[0u8; 2], reader)?),
                9 | 10 | 11 => {
                    let class_index: u16 = read(&[0u8; 2], reader)?;
                    let name_and_type_index: u16 = read(&[0u8; 2], reader)?;

                    match tag {
                        9 => Entry::FieldRef(class_index, name_and_type_index),
                        10 => Entry::MethodRef(class_index, name_and_type_index),
                        _ => Entry::InterfaceMethodRef(class_index, name_and_type_index),
                    }
                }
                _ => unreachable!(),
            };
        }

        Ok(pool)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constant_pool() -> Result<(), ConstantPoolError> {
        let mut pool = ConstantPool::new();

        pool.push(ConstantPoolEntry::Utf8("hello world")); // 1
        pool.push(ConstantPoolEntry::Integer(1i32)); // 2
        pool.push(ConstantPoolEntry::Long(2i64)); // 3 - 4
        pool.push(ConstantPoolEntry::Double(f64::EPSILON)); // 5 - 6
        pool.push(ConstantPoolEntry::Class(1)); // 7 
        pool.push(ConstantPoolEntry::MethodRef(1, 7)); // 8
        pool.push(ConstantPoolEntry::FieldRef(1, 7)); // 9

        assert_eq!(pool.get(0).unwrap_err(), ConstantPoolError::InvalidIndex(0));
        assert_eq!(
            pool.get(10).unwrap_err(),
            ConstantPoolError::InvalidIndex(10)
        );

        assert_eq!(pool.get(4).unwrap_err(), ConstantPoolError::UnusableSlot(4));
        assert_eq!(pool.get(6).unwrap_err(), ConstantPoolError::UnusableSlot(6));

        assert_eq!(pool.get(1)?, &ConstantPoolEntry::Utf8("hello world"));
        assert_eq!(pool.get(8)?, &ConstantPoolEntry::MethodRef(1, 7));
        assert_eq!(pool.get(9)?, &ConstantPoolEntry::FieldRef(1, 7));

        Ok(())
    }
}
