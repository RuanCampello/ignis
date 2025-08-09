//! Field JVM representation.
//! A `field_info` structure is used to represent a field (instance variable or class variable) in a Java class.

use super::attributes::Attribute;
use crate::classfile::{ConstantPool, get_attributes, read};
use bitflags::bitflags;
use bumpalo::{Bump, collections::Vec};
use std::io::{BufReader, Read};

/// `field_info` defined by JVSM 4.5.
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub(in crate::classfile) struct Field<'at> {
    pub access_flags: FieldFlags,
    pub name_index: u16,
    pub descriptor_index: u16,
    pub attributes: &'at [Attribute<'at>],
}

bitflags! {
    #[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
    pub(crate) struct FieldFlags: u16 {
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

fn parse_fields<'c>(
    reader: &mut BufReader<impl Read>,
    count: usize,
    constant_pool: &'c ConstantPool<'c>,
    arena: &'c Bump,
) -> Result<&'c [crate::classfile::fields::Field<'c>], crate::classfile::ClassfileError> {
    let mut fields_vec = Vec::with_capacity_in(count, arena);
    for _ in 0..count {
        fields_vec.push(crate::classfile::fields::Field {
            access_flags: crate::classfile::fields::FieldFlags::from_bits_truncate(read(reader)?),
            name_index: read(reader)?,
            descriptor_index: read(reader)?,
            attributes: get_attributes(reader, constant_pool, arena)?,
        });
    }
    Ok(fields_vec.into_bump_slice())
}
