//! The `attributes` in a `field_info` structure provide additional metadata about a given field.
//! Those attributes are stored into an array of `attributes`, duh.

use super::{ClassfileError, constant_pool::ConstantPool};
use crate::classfile::{
    constant_pool::{ConstantPoolEntry, ConstantPoolError},
    read,
};
use bitflags::bitflags;
use bumpalo::collections::Vec;
use std::io::{BufReader, Read};
use thiserror::Error;

/// Attributes as defined by JSVM (4.7)
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub(in crate::classfile) enum Attribute<'at> {
    ConstantValue {
        constantvalue_index: u16,
    },
    Code {
        max_stack: u16,
        max_locals: u16,
        code: &'at [u8],
        exception_table: &'at [ExceptionEntry],
        attributes: &'at [Attribute<'at>],
    },
    StackMapTable {
        entries: &'at [StackMapEntry<'at>],
    },
    Exceptions {
        exception_index_table: &'at [u16],
    },
    InnerClasses {
        classes: &'at [InnerClassEntry],
    },
    EnclosingMethod {
        class_index: u16,
        method_index: u16,
    },
    Synthetic,
    Signature {
        signature_index: u16,
    },
    SourceFile {
        sourcefile_index: u16,
    },
    SourceDebugExtension,
    LineNumberTable {
        line_number_table: &'at [LineNumberEntry],
    },
    LocalVariableTable {
        local_variable_table: &'at [LocalVariableEntry],
    },
    LocalVariableTypeTable {
        local_variable_type_table: &'at [LocalVariableTypeEntry],
    },
    Deprecated,
    RuntimeVisibleAnnotations {
        bytes: &'at [u8],
        annotations: &'at [Annotation<'at>],
    },
    RuntimeInvisibleAnnotations {
        annotations: &'at [Annotation<'at>],
    },
    RuntimeVisibleParameterAnnotations,
    RuntimeInvisibleParameterAnnotations,
    RuntimeVisibleTypeAnnotations,
    RuntimeInvisibleTypeAnnotations,

    AnnotationDefault {
        element_value: ElementValue<'at>,
        bytes: &'at [u8],
    },
    BootstrapMethods,
    MethodParameters {
        parameters: &'at [MethodParameterEntry],
    },
    Module,
    ModulePackages,
    ModuleMainClass,
    NestHost {
        host_class_index: u16,
    },
    NestMembers {
        classes: &'at [u16],
    },
    Record {
        components: &'at [RecordComponentInfo<'at>],
    },
    PermittedSubclasses,
}

/// `element_value` structure as defined by JSVM (4.7.16.1)
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub(in crate::classfile) enum ElementValue<'at> {
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
        annotation_value: Annotation<'at>,
    },
    ArrayValue {
        tag: u8,
        values: &'at [ElementValue<'at>],
    },
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub(in crate::classfile) struct ExceptionEntry {
    start_pc: u16,
    end_pc: u16,
    handler_pc: u16,
    catch_type: u16,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub(in crate::classfile) enum StackMapEntry<'st> {
    SameFrame {
        offset_delta: u16,
    },
    SameStack {
        offset_delta: u16,
        stack: VerificationTypeInfo,
    },
    SameStackExtended {
        offset_delta: u16,
        stack: VerificationTypeInfo,
    },
    ChopFrame {
        offset_delta: u16,
    },
    SameFrameExtended {
        offset_delta: u16,
    },
    AppendFrame {
        offset_delta: u16,
        locals: &'st [VerificationTypeInfo],
    },
    FullFrame {
        offset_delta: u8,
        locals: &'st [VerificationTypeInfo],
        stack: &'st [VerificationTypeInfo],
    },
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub(in crate::classfile) struct InnerClassEntry {
    inner_class_info_index: u16,
    outer_class_info_index: u16,
    inner_name_index: u16,
    inner_class_access_flags: InnerClassFlags,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub(in crate::classfile) struct LineNumberEntry {
    start_pc: u16,
    line_number: u16,
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

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub(in crate::classfile) struct Annotation<'el> {
    type_index: u16,
    element_value_pairs: &'el [ElementValuePair<'el>],
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub(in crate::classfile) struct MethodParameterEntry {
    name_index: u16,
    access_flags: MethodParameterFlags,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub(in crate::classfile) struct ElementValuePair<'el> {
    element_name_index: u16,
    element_value: ElementValue<'el>,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub(in crate::classfile) struct RecordComponentInfo<'at> {
    name_index: u16,
    descriptor_index: u16,
    attributes: &'at [Attribute<'at>],
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
#[repr(u8)]
pub(in crate::classfile) enum FrameType {
    SameFrame,
    SameStack,
    SameStackExtended,
    ChopFrame { k: u8 },
    SameFrameExtended,
    AppendFrame { k: u8 },
    FullFrame,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
#[repr(u8)]
pub(in crate::classfile) enum VerificationTypeInfo {
    TopVariable,
    IntegerVariable,
    FloatVariable,
    LongVariable,
    DoubleVariable,
    NullVariable,
    UninitializedThisVariable,
    ObjectVariable { cpool_index: u16 },
    UninitializedVariable { offset: u16 },
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

bitflags! {
    #[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
    pub struct MethodParameterFlags: u16 {
        /// Indicates that the formal parameter was declared `final`.
        const ACC_FINAL =     0x0010;
        /// Indicates that the formal parameter was not explicitly or implicitly declared in source code,
        /// according to the specification of the language in which the source code was written (JLS §13.1).
        /// (The formal parameter is an implementation artifact of the compiler which produced this class file.)
        const ACC_SYNTHETIC = 0x1000;
        /// Indicates that the formal parameter was implicitly declared in source code,
        /// according to the specification of the language in which the source code was written (JLS §13.1).
        /// (The formal parameter is mandated by a language specification, so all compilers for the language must emit it.)
        const ACC_MANDATED =  0x8000;
    }
}

impl<'at> AsRef<Attribute<'at>> for Attribute<'at> {
    fn as_ref(&self) -> &Attribute<'at> {
        self
    }
}

impl<'at> Attribute<'at> {
    fn new<'pool>(
        reader: &mut BufReader<impl Read>,
        name_index: u16,
        length: u32,
        constant_pool: &'at ConstantPool<'at>,
        arena: &'at bumpalo::Bump,
    ) -> Result<Self, ClassfileError> {
        let attribute_name: &str = constant_pool.get_with(name_index, |entry| match entry {
            ConstantPoolEntry::Utf8(utf8) => Ok(utf8),
            attr => panic!(
                "Attribute {attr:?} with index {name_index} is not a Utf8 entry in the constant pool."
            ),
        })?;

        let attribute = match attribute_name {
            "ConstantValue" => Attribute::ConstantValue {
                constantvalue_index: read(reader)?,
            },

            "Code" => {
                let max_stack: u16 = read(reader)?;
                let max_locals: u16 = read(reader)?;
                let code_len: u32 = read(reader)?;

                let mut code = bumpalo::vec![in arena; 0; code_len as usize];
                reader.read_exact(&mut code)?;
                let code = code.into_bump_slice();

                let expection_table_len: u16 = read(reader)?;
                let mut exception_table =
                    Vec::with_capacity_in(expection_table_len as usize, arena);
                for _ in (0..expection_table_len) {
                    exception_table.push(ExceptionEntry {
                        start_pc: read::<u16>(reader)?,
                        end_pc: read::<u16>(reader)?,
                        handler_pc: read::<u16>(reader)?,
                        catch_type: read::<u16>(reader)?,
                    });
                }

                let attributes = get_attributes(reader, constant_pool, arena)?;
                Attribute::Code {
                    max_stack,
                    max_locals,
                    code,
                    exception_table: exception_table.into_bump_slice(),
                    attributes,
                }
            }

            "StackMapTable" => {
                let stack_map_table_entries = read::<u16>(reader)? as usize;
                let mut entries = Vec::with_capacity_in(stack_map_table_entries, arena);

                for _ in (0..stack_map_table_entries) {
                    let frame_byte: u8 = read(reader)?;
                    let frame_type = FrameType::from(frame_byte);

                    let entry = match frame_type {
                        FrameType::SameFrame => StackMapEntry::SameFrame {
                            offset_delta: frame_byte as _,
                        },

                        FrameType::SameStack => {
                            let offset_delta = frame_byte as u16 - 64;
                            let stack = VerificationTypeInfo::try_from(&mut *reader)?;

                            StackMapEntry::SameStack {
                                offset_delta,
                                stack,
                            }
                        }

                        FrameType::SameStackExtended => {
                            let stack = VerificationTypeInfo::try_from(&mut *reader)?;

                            StackMapEntry::SameStackExtended {
                                offset_delta: frame_byte as _,
                                stack,
                            }
                        }

                        FrameType::ChopFrame { .. } => StackMapEntry::ChopFrame {
                            offset_delta: read(reader)?,
                        },

                        FrameType::SameFrameExtended => StackMapEntry::SameFrameExtended {
                            offset_delta: read(reader)?,
                        },

                        FrameType::AppendFrame { k } => {
                            let offset_delta = read(reader)?;
                            let mut locals = Vec::with_capacity_in(k as usize, arena);
                            for _ in (0..k) {
                                locals.push(VerificationTypeInfo::try_from(&mut *reader)?);
                            }

                            StackMapEntry::AppendFrame {
                                offset_delta,
                                locals: locals.into_bump_slice(),
                            }
                        }

                        FrameType::FullFrame => {
                            let offset_delta = read(reader)?;
                            let locals_count = read::<u16>(reader)? as usize;
                            let mut locals = Vec::with_capacity_in(locals_count, arena);

                            for _ in (0..locals_count) {
                                locals.push(VerificationTypeInfo::try_from(&mut *reader)?);
                            }

                            let stack_count = read::<u16>(reader)? as usize;
                            let mut stack = Vec::with_capacity_in(stack_count, arena);

                            StackMapEntry::FullFrame {
                                offset_delta,
                                locals: locals.into_bump_slice(),
                                stack: stack.into_bump_slice(),
                            }
                        }
                    };

                    entries.push(entry)
                }

                Attribute::StackMapTable {
                    entries: entries.into_bump_slice(),
                }
            }

            "Exceptions" => {
                let exceptions_count: u16 = read(reader)?;

                let mut exception_index_table =
                    Vec::with_capacity_in(exceptions_count as usize, arena);

                for _ in (0..exceptions_count) {
                    exception_index_table.push(read::<u16>(reader)?)
                }

                Attribute::Exceptions {
                    exception_index_table: exception_index_table.into_bump_slice(),
                }
            }

            "InnerClasses" => {
                let number_of_classes: u16 = read(reader)?;
                let mut classes = Vec::with_capacity_in(number_of_classes as usize, arena);

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

                Attribute::InnerClasses {
                    classes: classes.into_bump_slice(),
                }
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

            "LineNumberTable" => {
                let line_number_table_count = read::<u16>(reader)? as usize;
                let mut line_number_table = Vec::with_capacity_in(line_number_table_count, arena);
                for _ in (0..line_number_table_count) {
                    let entry = LineNumberEntry {
                        start_pc: read(reader)?,
                        line_number: read(reader)?,
                    };

                    line_number_table.push(entry);
                }

                Attribute::LineNumberTable {
                    line_number_table: line_number_table.into_bump_slice(),
                }
            }

            "LocalVariableTable" => {
                let local_variable_table_count: u16 = read(reader)?;
                let mut local_variable_table =
                    Vec::with_capacity_in(local_variable_table_count as _, arena);

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
                    local_variable_table: local_variable_table.into_bump_slice(),
                }
            }
            "LocalVariableTypeTable" => {
                let local_variable_type_count: u16 = read(reader)?;
                let mut local_variable_type_table =
                    Vec::with_capacity_in(local_variable_type_count as _, arena);

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
                    local_variable_type_table: local_variable_type_table.into_bump_slice(),
                }
            }

            "RuntimeVisibleAnnotations" => {
                let mut bytes = bumpalo::vec![in arena; 0; length as usize];
                reader.read_exact(&mut bytes)?;
                let bytes = bytes.into_bump_slice();
                let mut reader = BufReader::new(&bytes[..]);

                let annotation_count = read::<u16>(&mut reader)? as usize;
                let mut annotations = Vec::with_capacity_in(annotation_count, arena);

                for _ in (0..annotation_count) {
                    annotations.push(get_annotation(&mut reader, constant_pool, arena)?);
                }

                Attribute::RuntimeVisibleAnnotations {
                    annotations: annotations.into_bump_slice(),
                    bytes,
                }
            }

            "RuntimeInvisibleAnnotations" => {
                let annotation_count = read::<u16>(reader)? as usize;
                let mut annotations = Vec::with_capacity_in(annotation_count, arena);

                for _ in (0..annotation_count) {
                    annotations.push(get_annotation(reader, constant_pool, arena)?);
                }

                Attribute::RuntimeInvisibleAnnotations {
                    annotations: annotations.into_bump_slice(),
                }
            }

            "AnnotationDefault" => {
                let mut bytes = bumpalo::vec![in arena; 0; length as usize];
                reader.read_exact(&mut bytes)?;
                let bytes = bytes.into_bump_slice();
                let mut reader = BufReader::new(&bytes[..]);

                Attribute::AnnotationDefault {
                    element_value: get_element_value(&mut reader, constant_pool, arena)?,
                    bytes,
                }
            }

            "MethodParameters" => {
                let parameter_count = read::<u8>(reader)? as usize;
                let mut parameters = Vec::with_capacity_in(parameter_count, arena);

                for _ in (0..parameter_count) {
                    parameters.push(MethodParameterEntry {
                        name_index: read(reader)?,
                        access_flags: MethodParameterFlags::from_bits_truncate(read(reader)?),
                    });
                }
                let parameters = parameters.into_bump_slice();
                Attribute::MethodParameters { parameters }
            }

            "NestHost" => Attribute::NestHost {
                host_class_index: read(reader)?,
            },

            "NestMembers" => {
                let classes_count = read::<u16>(reader)? as usize;
                let mut classes = Vec::with_capacity_in(classes_count, arena);

                for _ in (0..classes_count) {
                    classes.push(read(reader)?);
                }

                Attribute::NestMembers {
                    classes: classes.into_bump_slice(),
                }
            }

            "Record" => {
                let component_count = read::<u16>(reader)? as usize;
                let mut components = Vec::with_capacity_in(component_count, arena);

                for _ in (0..component_count) {
                    components.push(RecordComponentInfo {
                        name_index: read(reader)?,
                        descriptor_index: read(reader)?,
                        attributes: get_attributes(reader, constant_pool, arena)?,
                    })
                }

                Attribute::Record {
                    components: components.into_bump_slice(),
                }
            }
            _ => unimplemented!("Parsing for Attribute: {attribute_name} is not yet implemented"),
        };

        Ok(attribute)
    }
}

impl<R: Read> TryFrom<&mut BufReader<R>> for VerificationTypeInfo {
    type Error = ClassfileError;

    fn try_from(reader: &mut BufReader<R>) -> Result<Self, Self::Error> {
        let tag: u8 = read(reader)?;

        match tag {
            0 => Ok(VerificationTypeInfo::TopVariable),
            1 => Ok(VerificationTypeInfo::IntegerVariable),
            2 => Ok(VerificationTypeInfo::FloatVariable),
            3 => Ok(VerificationTypeInfo::DoubleVariable),
            4 => Ok(VerificationTypeInfo::LongVariable),
            5 => Ok(VerificationTypeInfo::NullVariable),
            6 => Ok(VerificationTypeInfo::UninitializedThisVariable),
            7 => {
                let cpool_index = read::<u16>(reader)?;
                Ok(VerificationTypeInfo::ObjectVariable { cpool_index })
            }
            8 => {
                let offset = read::<u16>(reader)?;
                Ok(VerificationTypeInfo::UninitializedVariable { offset })
            }
            _ => unreachable!("VerificationTypeInfo for tag: {tag} is not defined"),
        }
    }
}

impl From<u8> for FrameType {
    fn from(value: u8) -> Self {
        match value {
            0..=63 => Self::SameFrame,
            64..=127 => Self::SameStack,
            247 => Self::SameStackExtended,
            248..=250 => Self::ChopFrame { k: 251 - value },
            251 => Self::SameFrameExtended,
            252..=254 => Self::AppendFrame { k: value - 251 },
            255 => Self::FullFrame,
            _ => unreachable!("FrameType for '{value}' is not defined"),
        }
    }
}

pub(in crate::classfile) fn get_attributes<'at>(
    reader: &mut BufReader<impl Read>,
    constant_pool: &'at ConstantPool<'at>,
    arena: &'at bumpalo::Bump,
) -> Result<&'at [Attribute<'at>], ClassfileError> {
    let attributes_count: u16 = read(reader)?;
    let mut attributes =
        bumpalo::collections::Vec::with_capacity_in(attributes_count as usize, arena);

    for _ in 0..attributes_count {
        let name_index: u16 = read(reader)?;
        let length = read::<u32>(reader)?;

        let attribute = Attribute::new(reader, name_index, length, constant_pool, arena)?;
        attributes.push(attribute);
    }

    Ok(attributes.into_bump_slice())
}
fn get_annotation<'at>(
    reader: &mut BufReader<impl Read>,
    constant_pool: &'at ConstantPool<'at>,
    arena: &'at bumpalo::Bump,
) -> Result<Annotation<'at>, ClassfileError> {
    let type_index: u16 = read(reader)?;
    let num_element_pairs = read::<u16>(reader)? as usize;
    let mut element_value_pairs = Vec::with_capacity_in(num_element_pairs, arena);

    for _ in (0..num_element_pairs) {
        let element_name_index: u16 = read(reader)?;
        let element_value = get_element_value(reader, constant_pool, arena)?;

        element_value_pairs.push(ElementValuePair {
            element_name_index,
            element_value,
        })
    }

    Ok(Annotation {
        type_index,
        element_value_pairs: element_value_pairs.into_bump_slice(),
    })
}

fn get_element_value<'el>(
    reader: &mut BufReader<impl Read>,
    constant_pool: &'el ConstantPool,
    arena: &'el bumpalo::Bump,
) -> Result<ElementValue<'el>, ClassfileError> {
    let tag: u8 = read(reader)?;

    match tag {
        b'B' | b'C' | b'D' | b'F' | b'I' | b'J' | b'S' | b'Z' | b's' => {
            Ok(ElementValue::ConstValueIndex {
                tag,
                const_value_index: read(reader)?,
            })
        }

        b'e' => Ok(ElementValue::EnumConstValue {
            tag,
            type_name_index: read(reader)?,
            const_name_index: read(reader)?,
        }),

        b'c' => Ok(ElementValue::ClassInfoIndex {
            tag,
            class_info_index: read(reader)?,
        }),

        b'@' => Ok(ElementValue::Annotation {
            tag,
            annotation_value: get_annotation(reader, constant_pool, arena)?,
        }),

        b'[' => {
            let values_count = read::<u16>(reader)? as usize;
            let mut values = Vec::with_capacity_in(values_count, arena);

            for _ in (0..values_count) {
                values.push(get_element_value(reader, constant_pool, arena)?);
            }

            Ok(ElementValue::ArrayValue {
                tag,
                values: values.into_bump_slice(),
            })
        }

        _ => unreachable!("ElementValue with tag: '{tag}' is not applicable"),
    }
}
