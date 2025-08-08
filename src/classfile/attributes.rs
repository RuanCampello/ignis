//! The `attributes` in a `field_info` structure provide additional metadata about a given field.
//! Those attributes are stored into an array of `attributes`, duh.

use super::{ClassfileError, constant_pool::ConstantPool};
use crate::classfile::{
    constant_pool::{ConstantPoolEntry, ConstantPoolError},
    read, read_bytes,
};
use std::io::{BufReader, Read};
use thiserror::Error;

/// Attributes as defined by JSVM (4.7)
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
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
        exception_table: Vec<ExceptionEntry>,
        // PERFORMANCE: make this use references so we don't need to do heap allocation when moving this
        // around
        // exception_table: &'at [ExceptionEntry],
        // attributes: &'at [Attribute<'at>],
        attributes: Vec<Attribute<'at>>,
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

impl<'at> AsRef<Attribute<'at>> for Attribute<'at> {
    fn as_ref(&self) -> &Attribute<'at> {
        self
    }
}

impl<'at> TryFrom<(Vec<u8>, &'at ConstantPool<'_>)> for Attribute<'at> {
    type Error = ClassfileError;

    fn try_from(value: (Vec<u8>, &'at ConstantPool)) -> Result<Self, Self::Error> {
        let (buffer, constant_pool) = value;
        let reader = &mut BufReader::new(buffer.as_slice());
        let mut cursor = 0usize;

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
                let code = read_bytes(code_len as usize, reader, buffer.as_slice(), &mut cursor)?;

                let expection_table_len: u16 = read(reader)?;
                let mut exception_table = Vec::with_capacity(expection_table_len as usize);
                for _ in (0..expection_table_len) {
                    exception_table.push(ExceptionEntry {
                        start_pc: read::<u16>(reader)?,
                        end_pc: read::<u16>(reader)?,
                        handler_pc: read::<u16>(reader)?,
                        catch_type: read::<u16>(reader)?,
                    })
                }

                let attributes = get_attributes(reader, constant_pool)?;
                Attribute::Code {
                    max_stack,
                    max_locals,
                    code,
                    exception_table,
                    attributes,
                }
            }

            _ => todo!(),
        };

        todo!()
    }
}

fn get_attributes<'at>(
    reader: &mut BufReader<impl Read>,
    constant_pool: &'at ConstantPool,
) -> Result<Vec<Attribute<'at>>, ClassfileError> {
    let attributes_count: u16 = read(reader)?;
    let mut attributes = Vec::with_capacity(attributes_count as usize);

    for _ in (0..attributes_count) {
        let name_index: u16 = read(reader)?;
        let length = read::<u32>(reader)? as usize;

        let buffer = vec![0u8; length];
        let attribute = Attribute::try_from((buffer, constant_pool))?;
        attributes.push(attribute)
    }

    Ok(attributes)
}
