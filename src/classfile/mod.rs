//! This module is responsible for parsing and representing `.class` files as defined by the Java Virtual Machine specification.
//!
//! Which include things like:
//! - Low-level binary parsing of `.class` files, including constant pool, fields, methods, and attributes.
//! - Data structures to represent class file components in memory.
//! - Validation of class file format and version.
//!
//! The output of this module is a structured `ClassFile` representation, which is used by the class loader and interpreter.

mod attributes;
mod constant_pool;
mod fields;

use bitflags::bitflags;
use constant_pool::{ConstantPool, ConstantPoolError};
use fields::Field;
use std::io::{BufReader, Cursor, Read};
use thiserror::Error;

/// Classfile structure defined by JVMS (4.1)
#[derive(Debug, PartialEq, Clone)]
pub(crate) struct Classfile<'p> {
    version: Version,
    constant_pool: ConstantPool<'p>,
    access_flags: AccessFlags,
    this_class: u16,
    super_class: u16,
    interfaces: Vec<u16>,
    fields: Vec<Field>,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
/// This determines the version of the `class` file format.
pub(crate) struct Version {
    major: u16,
    minor: u16,
}

#[derive(Error, Debug)]
pub(crate) enum ClassfileError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("Invalid classfile: magic number doesn't match.")]
    InvalidClassfile,
    #[error("Invalid UTF-8 string: {0}")]
    InvalidUtf8(#[from] core::str::Utf8Error),
    #[error("Invalid or incompatible version found: {0}")]
    Version(u16),
    #[error(transparent)]
    ConstantPool(#[from] ConstantPoolError),
}

/// Magic header number for a `.class` file.
pub(crate) const MAGIC: u32 = 0xCAFEBABE;

bitflags! {
    /// Class, field, method, and module access and property flags
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub(crate) struct AccessFlags: u16 {
        /// Declared public; may be accessed from outside its package.
        const PUBLIC = 0x0001;
        /// Declared final; no subclasses allowed.
        const FINAL = 0x0010;
        /// Treat superclass methods specially when invoked by the invokespecial instruction.
        const SUPER = 0x0020;
        /// Is an interface, not a class.
        const INTERFACE = 0x0200;
        /// Declared abstract; must not be instantiated.
        const ABSTRACT = 0x0400;
        /// Declared synthetic; not present in the source code.
        const SYNTHETIC = 0x1000;
        /// Declared as an annotation interface.
        const ANNOTATION = 0x2000;
        /// Declared as an enum class.
        const ENUM = 0x4000;
        /// Is a module, not a class or interface.
        const MODULE = 0x8000;
    }
}

impl<'c> TryFrom<&[u8]> for Classfile<'c> {
    type Error = ClassfileError;

    fn try_from(buff: &[u8]) -> Result<Self, Self::Error> {
        let mut reader = BufReader::new(buff);

        let magic = read::<u32>(&buff, &mut reader)?;
        if magic != MAGIC {
            return Err(ClassfileError::InvalidClassfile);
        }

        let minor = read::<u16>(&buff, &mut reader)?;
        let major = read::<u16>(&buff, &mut reader)?;
        if !Version::is_valid(major) {
            return Err(ClassfileError::Version(major));
        }
        let version = Version::new(major, minor);

        todo!()
    }
}

impl Version {
    const fn new(major: u16, minor: u16) -> Self {
        Self { major, minor }
    }

    fn is_valid(major: u16) -> bool {
        (45..=68).contains(&major)
    }
}

pub(self) fn read<T>(buff: &[u8], reader: &mut BufReader<impl Read>) -> Result<T, ClassfileError>
where
    T: Sized + From<u8> + Copy,
    T: core::ops::Shl<u8, Output = T> + core::ops::BitOr<Output = T>,
{
    let size = size_of::<T>();
    let mut buff = vec![0u8; size];
    reader.read_exact(&mut buff)?;

    let mut value = T::from(buff[0]);

    &buff[1..]
        .iter()
        .for_each(|&byte| value = (value << 8.into()) | T::from(byte));

    Ok(value)
}
