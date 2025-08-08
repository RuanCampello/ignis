//! The `attributes` in a `field_info` structure provide additional metadata about a given field.
//! Those attributes are stored into an array of `attributes`, duh.

use super::{ClassfileError, constant_pool::ConstantPool};
use crate::classfile::{
    constant_pool::{ConstantPoolEntry, ConstantPoolError},
    read, read_bytes,
};
use bitflags::bitflags;
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
    Exceptions {
        exception_index_table: Vec<u16>,
    },
    InnerClasses {
        classes: Vec<InnerClassEntry>,
    },

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

    SourceFile {
        sourcefile_index: u16,
    },

    SourceDebugExtension,
    LineNumberTable,
    LocalVariableTable {
        local_variable_table: Vec<LocalVariableEntry>,
    },
    LocalVariableTypeTable {
        local_variable_type_table: Vec<LocalVariableTypeEntry>,
    },
    Deprecated,
    RuntimeVisibleAnnotations,
    RuntimeInvisibleAnnotations,
    RuntimeVisibleParameterAnnotations,
    RuntimeInvisibleParameterAnnotations,
    RuntimeVisibleTypeAnnotations,
    RuntimeInvisibleTypeAnnotations,
    AnnotationDefault {
        element_value: ElementValue,
        inner: Vec<u8>,
    },
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

/// `element_value` structure as defined by JSVM (4.7.16.1)
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub(in crate::classfile) enum ElementValue {
    ConstValueIndex {
        tag: u8,
        const_value_index: u16,
    },
    EnumConstValue {
        tag: u8,
        type_name_index: u16,
        const_name_index: u16,
    },
    ClassInfoIndex {
        tag: u8,
        class_info_index: u16,
    },
    Annotation {
        tag: u8,
        annotation_value: Annotation,
    },
    ArrayValue {
        tag: u8,
        array_value: Vec<ElementValue>,
    },
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub(in crate::classfile) struct ExceptionEntry {
    start_pc: u16,
    end_pc: u16,
    handler_pc: u16,
    catch_type: u16,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub(in crate::classfile) struct InnerClassEntry {
    inner_class_info_index: u16,
    outer_class_info_index: u16,
    inner_name_index: u16,
    inner_class_access_flags: InnerClassFlags,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub(in crate::classfile) struct LocalVariableEntry {
    start_pc: u16,
    length: u16,
    name_index: u16,
    descriptor_index: u16,
    index: u16,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub(in crate::classfile) struct LocalVariableTypeEntry {
    start_pc: u16,
    length: u16,
    name_index: u16,
    signature_index: u16,
    index: u16,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub(in crate::classfile) struct Annotation {
    type_index: u16,
    element_value_pairs: Vec<ElementValuePair>,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub(in crate::classfile) struct ElementValuePair {
    element_name_index: u16,
    element_value: ElementValue,
}

bitflags! {
    #[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
    // TODO: add documentation for this ones
    pub(crate) struct InnerClassFlags: u16 {
        const PUBLIC     = 0x0001;
        const PRIVATE    = 0x0002;
        const PROTECTED  = 0x0004;
        const STATIC     = 0x0008;
        const FINAL      = 0x0010;
        const INTERFACE  = 0x0200;
        const ABSTRACT   = 0x0400;
        const SYNTHETIC  = 0x1000;
        const ANNOTATION = 0x2000;
        const ENUM       = 0x4000;
    }
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

            "StackMapTable" => todo!(),

            "Exceptions" => {
                let exceptions_count: u16 = read(reader)?;

                let mut exception_index_table = Vec::with_capacity(exceptions_count as usize);

                for _ in (0..exceptions_count) {
                    exception_index_table.push(read::<u16>(reader)?)
                }

                Attribute::Exceptions {
                    exception_index_table,
                }
            }

            "InnerClasses" => {
                let number_of_classes: u16 = read(reader)?;
                let mut classes = Vec::with_capacity(number_of_classes as usize);

                for _ in (0..number_of_classes) {
                    let entry = InnerClassEntry {
                        inner_class_info_index: read(reader)?,
                        outer_class_info_index: read(reader)?,
                        inner_name_index: read(reader)?,
                        inner_class_access_flags: InnerClassFlags::from_bits_truncate(read(
                            reader,
                        )?),
                    };

                    classes.push(entry)
                }

                Attribute::InnerClasses { classes }
            }

            "EnclosingMethod" => {
                let class_index: u16 = read(reader)?;
                let method_index: u16 = read(reader)?;

                Attribute::EnclosingMethod {
                    class_index,
                    method_index,
                }
            }
            "Synthetic" => Attribute::Synthetic,
            "Deprecated" => Attribute::Deprecated,
            "Signature" => Attribute::Signature {
                signature_index: read::<u16>(reader)?,
            },
            "SourceFile" => Attribute::SourceFile {
                sourcefile_index: read::<u16>(reader)?,
            },
            "LocalVariableTable" => {
                let local_variable_table_count: u16 = read(reader)?;
                let mut local_variable_table = Vec::with_capacity(local_variable_table_count as _);

                for _ in (0..local_variable_table_count) {
                    let entry = LocalVariableEntry {
                        start_pc: read(reader)?,
                        length: read(reader)?,
                        name_index: read(reader)?,
                        descriptor_index: read(reader)?,
                        index: read(reader)?,
                    };

                    local_variable_table.push(entry)
                }

                Attribute::LocalVariableTable {
                    local_variable_table,
                }
            }
            "LocalVariableTypeTable" => {
                let local_variable_type_count: u16 = read(reader)?;
                let mut local_variable_type_table =
                    Vec::with_capacity(local_variable_type_count as _);

                for _ in (0..local_variable_type_count) {
                    let entry = LocalVariableTypeEntry {
                        start_pc: read(reader)?,
                        length: read(reader)?,
                        name_index: read(reader)?,
                        signature_index: read(reader)?,
                        index: read(reader)?,
                    };

                    local_variable_type_table.push(entry);
                }

                Attribute::LocalVariableTypeTable {
                    local_variable_type_table,
                }
            }
            "AnnotationDefault" => {
                todo!()
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

fn get_annotation(
    reader: &mut BufReader<impl Read>,
    constant_pool: &ConstantPool,
) -> Result<Annotation, ClassfileError> {
    let type_index: u16 = read(reader)?;
    let num_element_pairs = read::<u16>(reader)? as usize;
    let mut element_value_pairs = Vec::with_capacity(num_element_pairs);

    for _ in (0..num_element_pairs) {
        let element_name_index: u16 = read(reader)?;
        let element_value = get_element_value(reader, constant_pool)?;

        element_value_pairs.push(ElementValuePair {
            element_name_index,
            element_value,
        })
    }

    Ok(Annotation {
        type_index,
        element_value_pairs,
    })
}

fn get_element_value(
    reader: &mut BufReader<impl Read>,
    constant_pool: &ConstantPool,
) -> Result<ElementValue, ClassfileError> {
    unimplemented!()
}
