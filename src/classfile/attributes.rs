//! The `attributes` in a `field_info` structure provide additional metadata about a given field.
//! Those attributes are stored into an array of `attributes`, duh.

/// Attributes as defined by JSVM (4.7)
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub(crate) enum Attribute {
    /// JSVM (4.7.2)
    ConstantValue {
        constantvalue_index: u16,
    },

    /// JSVM (4.7.3)
    Code {
        max_stack: u16,
        max_locals: u16,
        code: Vec<u8>,
        exception_table: Vec<ExceptionEntry>,
        attributes: Vec<Attribute>,
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
