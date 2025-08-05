//! Field JVM representation.
//! A `field_info` structure is used to represent a field (instance variable or class variable) in a Java class.

use bitflags::bitflags;

/// `field_info` defined by JVSM 4.5.
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub(crate) struct Field {
    access_flags: u8,
    name_index: u16,
    descriptor_index: u16,
    attributes_count: u16,
}

bitflags! {
    #[derive(Debug, Clone, Copy)]
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
