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

use constant_pool::ConstantPool;
use fields::Field;

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

impl Version {
    pub(crate) fn is_valid(major: u16) -> bool {
        (45..=68).contains(&major)
    }
}
