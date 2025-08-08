//! The `attributes` in a `field_info` structure provide additional metadata about a given field.
//! Those attributes are stored into an array of `attributes`, duh.

use super::{ClassfileError, constant_pool::ConstantPool};
use crate::classfile::{
    constant_pool::{ConstantPoolEntry, ConstantPoolError},
    read, read_bytes,
};
use std::io::BufReader;
use thiserror::Error;

/// Attributes as defined by JSVM (4.7)
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub(in crate::classfile) enum Attribute<'at> {
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
pub(in crate::classfile) struct ExceptionEntry {
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

        let attribute_name_index: u16 = read(reader)?;
        let attribute_name: &str =
            constant_pool.get_with(attribute_name_index, |entry| match entry {
                ConstantPoolEntry::Utf8(utf8) => Ok(utf8),
                _ => Err(ConstantPoolError::InvalidAttr(
                    attribute_name_index as usize,
                )),
            })?;

        let attribute_len: u32 = read(reader)?;
        let attribute = match attribute_name {
            "ConstantValue" => Attribute::ConstantValue {
                constantvalue_index: read(reader)?,
            },
            "Code" => {
                let max_stack: u16 = read(reader)?;
                let max_locals: u16 = read(reader)?;
                let code_len: u32 = read(reader)?;
                let code = read_bytes(code_len as usize, reader)?.as_slice();

                let expection_table_len: u16 = read(reader)?;
                let mut expection_table = Vec::with_capacity(expection_table_len as usize);
                for _ in (0..expection_table_len) {
                    expection_table.push(ExceptionEntry {
                        start_pc: read::<u16>(reader)?,
                        end_pc: read::<u16>(reader)?,
                        handler_pc: read::<u16>(reader)?,
                        catch_type: read::<u16>(reader)?,
                    })
                }

                let attributes_count: u16 = read(reader)?;

                todo!()
            }

            _ => todo!(),
        };

        todo!()
    }
}
