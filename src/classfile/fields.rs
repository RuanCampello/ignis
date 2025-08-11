//! Field JVM representation.
//! A `field_info` structure is used to represent a field (instance variable or class variable) in a Java class.

use super::attributes::Attribute;
use crate::classfile::{ClassfileError, ConstantPool, get_attributes, read};
use bitflags::bitflags;
use bumpalo::{Bump, collections::Vec};
use std::io::{BufReader, Read};

/// `field_info` defined by JVSM 4.5.
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct Field<'at> {
    pub(super) access_flags: FieldFlags,
    pub(super) name_index: u16,
    pub(super) descriptor_index: u16,
    pub(super) attributes: &'at [Attribute<'at>],
}

bitflags! {
    #[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
    pub struct FieldFlags: u16 {
        /// Declared public; may be accessed from outside its package.
        const PUBLIC    = 0x0001;
        /// Declared private; accessible only within the defining class and other classes belonging
        /// to the same nest.
        const PRIVATE   = 0x0002;
        /// Declared protected; may be accessed within subclasses.
        const PROTECTED = 0x0004;
        /// Declared static.
        const STATIC    = 0x0008;
        /// Declared final; never directly assigned to after object construction.
        const FINAL     = 0x0010;
        /// Declared volatile; cannot be cached.
        const VOLATILE  = 0x0040;
        /// Declared transient; not written or read by a persistent object manager.
        const TRANSIENT = 0x0080;
        /// Declared synthetic; not present in the source code.
        const SYNTHETIC = 0x1000;
        /// Declared as an element of an enum class.
        const ENUM      = 0x4000;
    }
}

impl<'f> Field<'f> {
    pub fn contains(&self, flag: FieldFlags) -> bool {
        self.access_flags.contains(flag)
    }
}

pub(in crate::classfile) fn parse_fields<'c>(
    reader: &mut BufReader<impl Read>,
    constant_pool: &'c ConstantPool<'c>,
    arena: &'c Bump,
) -> Result<&'c [Field<'c>], ClassfileError> {
    let fields_count = read::<u16>(reader)? as usize;
    let mut fields_vec = Vec::with_capacity_in(fields_count, arena);

    for _ in (0..fields_count) {
        let entry = Field {
            access_flags: FieldFlags::from_bits_truncate(read(reader)?),
            name_index: read(reader)?,
            descriptor_index: read(reader)?,
            attributes: get_attributes(reader, constant_pool, arena)?,
        };

        fields_vec.push(entry);
    }

    Ok(fields_vec.into_bump_slice())
}
