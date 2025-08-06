//! The `attributes` in a `field_info` structure provide additional metadata about a given field.
//! Those attributes are stored into an array of `attributes`, duh.

use super::{ClassfileError, constant_pool::ConstantPool};
use crate::classfile::{
    constant_pool::{ConstantPoolEntry, ConstantPoolError},
    read,
};
use std::io::BufReader;
use thiserror::Error;

/// Attributes as defined by JSVM (4.7)
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub(crate) enum Attribute<'at> {
    /// JSVM (4.7.2)
    ConstantValue {
        constantvalue_index: u16,
    },

    /// JSVM (4.7.3)
    Code {
        max_stack: u16,
        max_locals: u16,
        code: &'at [u8],
        exception_table: &'at [ExceptionEntry],
        attributes: &'at [Attribute<'at>],
    },

    /// JSVM (4.7.4)
    StackMapTable,

    /// JSVM (4.7.5)
    Exceptions,
    InnerClasses,

    /// JSVM (4.7.7)
    EnclosingMethod {
        class_index: u16,
        method_index: u16,
    },

    /// JSVM (4.7.8)
    Synthetic,

    /// JSVM (4.7.9)
    Signature {
        signature_index: u16,
    },

    SourceFile,
    SourceDebugExtension,
    LineNumberTable,
    LocalVariableTable,
    LocalVariableTypeTable,
    Deprecated,
    RuntimeVisibleAnnotations,
    RuntimeInvisibleAnnotations,
    RuntimeVisibleParameterAnnotations,
    RuntimeInvisibleParameterAnnotations,
    RuntimeVisibleTypeAnnotations,
    RuntimeInvisibleTypeAnnotations,
    AnnotationDefault,
    BootstrapMethods,
    MethodParameters,
    Module,
    ModulePackages,
    ModuleMainClass,
    NestHost,
    NestMembers,
    Record,
    PermittedSubclasses,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub(crate) struct ExceptionEntry {
    start_pc: u16,
    end_pc: u16,
    handler_pc: u16,
    catch_type: u16,
}

impl<'at> TryFrom<(&[u8], ConstantPool<'_>)> for Attribute<'at> {
    type Error = ClassfileError;

    fn try_from(value: (&[u8], ConstantPool)) -> Result<Self, Self::Error> {
        let (buffer, constant_pool) = value;
        let reader = &mut BufReader::new(buffer);

        let attribute_name_index: u16 = read(&[0u8; 2], reader)?;
        let attribute_name = constant_pool.get_with(attribute_name_index, |entry| match entry {
            ConstantPoolEntry::Utf8(utf8) => Ok(utf8),
            _ => Err(ConstantPoolError::InvalidAttr(
                attribute_name_index as usize,
            )),
        })?;

        let attribute_len: u32 = read(&[0u8; 4], reader)?;

        todo!()
    }
}
