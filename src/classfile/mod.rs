//! This module is responsible for parsing and representing `.class` files as defined by the Java Virtual Machine specification.
//!
//! Which include things like:
//! - Low-level binary parsing of `.class` files, including constant pool, fields, methods, and attributes.
//! - Data structures to represent class file components in memory.
//! - Validation of class file format and version.
//!
//! The output of this module is a structured `ClassFile` representation, which is used by the class loader and interpreter.

#![allow(elided_named_lifetimes)]

mod attributes;
mod constant_pool;
mod fields;

use self::{attributes::get_attributes, fields::FieldFlags};
use bitflags::bitflags;
use bumpalo::{Bump, collections::Vec};
use constant_pool::{ConstantPool, ConstantPoolError};
use fields::Field;
use std::io::{BufReader, Cursor, Read};
use thiserror::Error;

/// Classfile structure defined by JVMS (4.1)
#[derive(Debug, PartialEq, Clone)]
pub struct Classfile<'p> {
    version: Version,
    constant_pool: ConstantPool<'p>,
    access_flags: AccessFlags,
    this_class: u16,
    super_class: u16,
    interfaces: &'p [u16],
    fields: &'p [Field<'p>],
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
/// This determines the version of the `class` file format.
pub(crate) struct Version {
    major: u16,
    minor: u16,
}

#[derive(Error, Debug)]
pub enum ClassfileError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("Invalid classfile: magic number doesn't match.")]
    InvalidClassfile,
    #[error("Invalid UTF-8 string: {0}")]
    InvalidUtf8(#[from] cesu8::Cesu8DecodingError),
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

trait FromBeBytes {
    type Bytes: Sized;
    fn from_be_bytes(bytes: Self::Bytes) -> Self;
}

macro_rules! impl_from_be_bytes {
    ($($t:ty),* $(,)?) => {
        $(
            impl FromBeBytes for $t {
                type Bytes = [u8; core::mem::size_of::<$t>()];
                fn from_be_bytes(bytes: Self::Bytes) -> Self {
                    <$t>::from_be_bytes(bytes)
                }
            }
        )*
    };
}

impl_from_be_bytes!(u8, u16, u32, u64, i8, i16, i32, i64, f32, f64);

impl<'c> Classfile<'c> {
    pub fn new<'b>(buff: &'b [u8], arena: &'c Bump) -> Result<Self, ClassfileError>
    where
        'b: 'c,
    {
        let mut reader = BufReader::new(buff);

        let magic = read::<u32>(&mut reader)?;
        if magic != MAGIC {
            return Err(ClassfileError::InvalidClassfile);
        }

        let minor = read::<u16>(&mut reader)?;
        let major = read::<u16>(&mut reader)?;
        if !Version::is_valid(major) {
            return Err(ClassfileError::Version(major));
        }
        let version = Version::new(major, minor);

        let constant_pool = ConstantPool::new(&mut reader, arena)?;
        let access_flags = AccessFlags::from_bits_truncate(read::<u16>(&mut reader)?);
        let this_class: u16 = read(&mut reader)?;
        let super_class: u16 = read(&mut reader)?;

        let mut interfaces = Vec::with_capacity_in(read::<u16>(&mut reader)? as usize, arena);
        for _ in (0..interfaces.len()) {
            interfaces.push(read::<u16>(&mut reader)?);
        }
        let interfaces: &'c [u16] = interfaces.into_bump_slice();

        let mut fields = Vec::with_capacity_in(read::<u16>(&mut reader)? as usize, arena);
        let fields_count = read::<u16>(&mut reader)? as usize;
        for _ in (0..fields_count) {
            fields.push(Field {
                access_flags: FieldFlags::from_bits_truncate(read(&mut reader)?),
                name_index: read(&mut reader)?,
                descriptor_index: read(&mut reader)?,
                attributes: get_attributes(&mut reader, &constant_pool, arena)?,
            })
        }
        let fields: &'c [Field<'c>] = fields.into_bump_slice();

        Ok(Classfile {
            version,
            constant_pool,
            access_flags,
            this_class,
            super_class,
            interfaces,
            fields,
        })
    }

    pub fn is_public(&self) -> bool {
        self.access_flags.contains(AccessFlags::PUBLIC)
    }
    pub fn is_final(&self) -> bool {
        self.access_flags.contains(AccessFlags::FINAL)
    }
    pub fn is_abstract(&self) -> bool {
        self.access_flags.contains(AccessFlags::ABSTRACT)
    }

    pub fn version(&self) -> (u16, u16) {
        (self.version.major, self.version.minor)
    }

    pub fn class_name(&self) -> Option<&str> {
        self.constant_pool.get_classname(self.this_class).ok()
    }

    pub fn super_class(&self) -> Option<&str> {
        self.constant_pool.get_classname(self.super_class).ok()
    }

    pub fn field_names(&'c self, arena: &'c Bump) -> Result<Vec<&'c str>, ConstantPoolError> {
        use self::constant_pool::ConstantPoolEntry;

        let mut names = bumpalo::collections::Vec::new_in(arena);

        for f in self.fields.iter() {
            let name = self.constant_pool.get_with(f.name_index, |e| match e {
                ConstantPoolEntry::Utf8(s) => Ok(*s),
                _ => Err(ConstantPoolError::InvalidIndex(f.name_index)),
            })?;
            names.push(name);
        }

        Ok(names)
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

pub(self) fn read<T>(reader: &mut impl Read) -> Result<T, ClassfileError>
where
    T: FromBeBytes,
    T::Bytes: AsMut<[u8]> + Default,
{
    let mut bytes = T::Bytes::default();
    reader.read_exact(bytes.as_mut())?;

    Ok(T::from_be_bytes(bytes))
}

pub(self) fn read_bytes<'b>(
    size: usize,
    reader: &mut impl Read,
    buffer: &'b [u8],
    pos: &mut usize,
) -> Result<&'b [u8], ClassfileError> {
    let start = *pos;
    let end = start + size;
    *pos = end;
    Ok(&buffer[start..end])
}
