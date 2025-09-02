//! Java bytecode opcode [instructions](https://docs.oracle.com/javase/specs/jvms/se24/html/jvms-6.html) definition.

#[derive(Debug, Default, PartialEq, Eq, Clone, Copy, num_enum::FromPrimitive)]
#[repr(u8)]
#[allow(non_camel_case_types)]
pub(crate) enum Opcode {
    // constants-related instructions
    /// Do nothing; execution proceeds to the next instruction.
    #[default]
    NOP = 0x0,
    /// Push the `null` object reference onto the operand stack.
    ACONST_NULL,
    /// Push the integer constant `-1` onto the operand stack.
    ICONST_M1,
    /// Push the integer constant `0` onto the operand stack.
    ICONST_0,
    /// Push the integer constant `1` onto the operand stack.
    ICONST_1,
    /// Push the integer constant `2` onto the operand stack.
    ICONST_2,
    /// Push the integer constant `3` onto the operand stack.
    ICONST_3,
    /// Push the integer constant `4` onto the operand stack.
    ICONST_4,
    /// Push the integer constant `5` onto the operand stack.
    ICONST_5,
    /// Push the long integer constant `0L` onto the operand stack.
    LCONST_0,
    /// Push the long integer constant `1L` onto the operand stack.
    LCONST_1,
    /// Push the `float` constant `0.0f` onto the operand stack.
    FCONST_0,
    /// Push the `float` constant `1.0f` onto the operand stack.
    FCONST_1,
    /// Push the `float` constant `2.0f` onto the operand stack.
    FCONST_2,
    /// Push the `double` constant `0.0` onto the operand stack.
    DCONST_0,
    /// Push the `double` constant `1.0` onto the operand stack.
    DCONST_1,
    /// Push a byte (sign-extended to an int) from the immediate operand onto the operand stack.
    BIPUSH,
    /// Push a short (sign-extended to an int) from the immediate operand onto the operand stack.
    SIPUSH,
    /// Push a constant from the runtime constant pool (index is a single byte) onto the operand stack.
    LDC,
    /// Push a constant from the runtime constant pool (index is two bytes) onto the operand stack.
    LDC_W,
    /// Push a `long` or `double` constant from the runtime constant pool (index is two bytes) onto the operand stack.
    LDC2_W,

    // load instructions
    /// Load an `int` value from a local variable onto the operand stack.
    ILOAD,
    /// Load a `long` value from a local variable onto the operand stack.
    LLOAD,
    /// Load a `float` value from a local variable onto the operand stack.
    FLOAD,
    /// Load a `double` value from a local variable onto the operand stack.
    DLOAD,
    /// Load an object reference from a local variable onto the operand stack.
    ALOAD,
    /// Load an `int` from local variable `0` onto the operand stack.
    ILOAD_0,
    /// Load an `int` from local variable `1` onto the operand stack.
    ILOAD_1,
    /// Load an `int` from local variable `2` onto the operand stack.
    ILOAD_2,
    /// Load an `int` from local variable `3` onto the operand stack.
    ILOAD_3,
    /// Load a `long` from local variable `0` onto the operand stack.
    LLOAD_0,
    /// Load a `long` from local variable `1` onto the operand stack.
    LLOAD_1,
    /// Load a `long` from local variable `2` onto the operand stack.
    LLOAD_2,
    /// Load a `long` from local variable `3` onto the operand stack.
    LLOAD_3,
    /// Load a `float` from local variable `0` onto the operand stack.
    FLOAD_0,
    /// Load a `float` from local variable `1` onto the operand stack.
    FLOAD_1,
    /// Load a `float` from local variable `2` onto the operand stack.
    FLOAD_2,
    /// Load a `float` from local variable `3` onto the operand stack.
    FLOAD_3,
    /// Load a `double` from local variable `0` onto the operand stack.
    DLOAD_0,
    /// Load a `double` from local variable `1` onto the operand stack.
    DLOAD_1,
    /// Load a `double` from local variable `2` onto the operand stack.
    DLOAD_2,
    /// Load a `double` from local variable `3` onto the operand stack.
    DLOAD_3,
    /// Load an object reference from local variable `0` onto the operand stack.
    ALOAD_0,
    /// Load an object reference from local variable `1` onto the operand stack.
    ALOAD_1,
    /// Load an object reference from local variable `2` onto the operand stack.
    ALOAD_2,
    /// Load an object reference from local variable `3` onto the operand stack.
    ALOAD_3,
    /// Load an `int` from an array onto the operand stack.
    IALOAD,
    /// Load a `long` from an array onto the operand stack.
    LALOAD,
    /// Load a `float` from an array onto the operand stack.
    FALOAD,
    /// Load a `double` from an array onto the operand stack.
    DALOAD,
    /// Load an object reference from an array onto the operand stack.
    AALOAD,
    /// Load a `byte` or `boolean` from an array onto the operand stack.
    BALOAD,
    /// Load a `char` from an array onto the operand stack.
    CALOAD,
    /// Load a `short` from an array onto the operand stack.
    SALOAD,

    ISTORE,
    LSTORE,
    FSTORE,
    DSTORE,
    ASTORE,
    ISTORE_0,
    ISTORE_1,
    ISTORE_2,
    ISTORE_3,
    LSTORE_0,
    LSTORE_1,
    LSTORE_2,
    LSTORE_3,
    FSTORE_0,
    FSTORE_1,
    FSTORE_2,
    FSTORE_3,
    DSTORE_0,
    DSTORE_1,
    DSTORE_2,
    DSTORE_3,
    ASTORE_0,
    ASTORE_1,
    ASTORE_2,
    ASTORE_3,
    IASTORE,
    LASTORE,
    FASTORE,
    DASTORE,
    AASTORE,
    BASTORE,
    CASTORE,
    SASTORE,

    // stack
    POP,
    POP2,
    DUP,
    DUP_X1,
    DUP_X2,
    DUP2,
    DUP2_X1,
    DUP2_X2,
    SWAP,

    // math
    IADD,
    LADD,
    FADD,
    DADD,
    ISUB,
    LSUB,
    FSUB,
    DSUB,
    IMUL,
    LMUL,
    FMUL,
    DMUL,
    IDIV,
    LDIV,
    FDIV,
    DDIV,
    IREM,
    LREM,
    FREM,
    DREM,
    INEG,
    LNEG,
    FNEG,
    DNEG,
    ISHL,
    LSHL,
    ISHR,
    LSHR,
    IUSHR,
    LUSHR,
    IAND,
    LAND,
    IOR,
    LOR,
    IXOR,
    LXOR,
    IINC,

    // comparations
    LCMP = 148,
    FCMPL,
    FCMPG,
    DCMPL,
    DCMPG,
    IFEQ,
    IFNE,
    IFLT,
    IFGE,
    IFGT,
    IFLE,
    IF_ICMPEQ,
    IF_ICMPNE,
    IF_ICMPLT,
    IF_ICMPGE,
    IF_ICMPGT,
    IF_ICMPLE,
    IF_ACMPEQ,
    IF_ACMPNE,
}

impl std::fmt::Display for Opcode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            // constants
            Opcode::NOP => write!(f, "NOP"),
            Opcode::ACONST_NULL => write!(f, "ACONST_NULL"),
            Opcode::ICONST_M1 => write!(f, "ICONST_M1"),
            Opcode::ICONST_0 => write!(f, "ICONST_0"),
            Opcode::ICONST_1 => write!(f, "ICONST_1"),
            Opcode::ICONST_2 => write!(f, "ICONST_2"),
            Opcode::ICONST_3 => write!(f, "ICONST_3"),
            Opcode::ICONST_4 => write!(f, "ICONST_4"),
            Opcode::ICONST_5 => write!(f, "ICONST_5"),
            Opcode::LCONST_0 => write!(f, "LCONST_0"),
            Opcode::LCONST_1 => write!(f, "LCONST_1"),
            Opcode::FCONST_0 => write!(f, "FCONST_0"),
            Opcode::FCONST_1 => write!(f, "FCONST_1"),
            Opcode::FCONST_2 => write!(f, "FCONST_2"),
            Opcode::DCONST_0 => write!(f, "DCONST_0"),
            Opcode::DCONST_1 => write!(f, "DCONST_1"),
            Opcode::BIPUSH => write!(f, "BIPUSH"),
            Opcode::SIPUSH => write!(f, "SIPUSH"),
            Opcode::LDC => write!(f, "LDC"),
            Opcode::LDC_W => write!(f, "LDC_W"),
            Opcode::LDC2_W => write!(f, "LDC2_W"),

            // loads
            Opcode::ILOAD => write!(f, "ILOAD"),
            Opcode::LLOAD => write!(f, "LLOAD"),
            Opcode::FLOAD => write!(f, "FLOAD"),
            Opcode::DLOAD => write!(f, "DLOAD"),
            Opcode::ALOAD => write!(f, "ALOAD"),
            Opcode::ILOAD_0 => write!(f, "ILOAD_0"),
            Opcode::ILOAD_1 => write!(f, "ILOAD_1"),
            Opcode::ILOAD_2 => write!(f, "ILOAD_2"),
            Opcode::ILOAD_3 => write!(f, "ILOAD_3"),
            Opcode::LLOAD_0 => write!(f, "LLOAD_0"),
            Opcode::LLOAD_1 => write!(f, "LLOAD_1"),
            Opcode::LLOAD_2 => write!(f, "LLOAD_2"),
            Opcode::LLOAD_3 => write!(f, "LLOAD_3"),
            Opcode::FLOAD_0 => write!(f, "FLOAD_0"),
            Opcode::FLOAD_1 => write!(f, "FLOAD_1"),
            Opcode::FLOAD_2 => write!(f, "FLOAD_2"),
            Opcode::FLOAD_3 => write!(f, "FLOAD_3"),
            Opcode::DLOAD_0 => write!(f, "DLOAD_0"),
            Opcode::DLOAD_1 => write!(f, "DLOAD_1"),
            Opcode::DLOAD_2 => write!(f, "DLOAD_2"),
            Opcode::DLOAD_3 => write!(f, "DLOAD_3"),
            Opcode::ALOAD_0 => write!(f, "ALOAD_0"),
            Opcode::ALOAD_1 => write!(f, "ALOAD_1"),
            Opcode::ALOAD_2 => write!(f, "ALOAD_2"),
            Opcode::ALOAD_3 => write!(f, "ALOAD_3"),
            Opcode::IALOAD => write!(f, "IALOAD"),
            Opcode::LALOAD => write!(f, "LALOAD"),
            Opcode::FALOAD => write!(f, "FALOAD"),
            Opcode::DALOAD => write!(f, "DALOAD"),
            Opcode::AALOAD => write!(f, "AALOAD"),
            Opcode::BALOAD => write!(f, "BALOAD"),
            Opcode::CALOAD => write!(f, "CALOAD"),
            Opcode::SALOAD => write!(f, "SALOAD"),

            // stores
            Opcode::ISTORE => write!(f, "ISTORE"),
            Opcode::LSTORE => write!(f, "LSTORE"),
            Opcode::FSTORE => write!(f, "FSTORE"),
            Opcode::DSTORE => write!(f, "DSTORE"),
            Opcode::ASTORE => write!(f, "ASTORE"),
            Opcode::ISTORE_0 => write!(f, "ISTORE_0"),
            Opcode::ISTORE_1 => write!(f, "ISTORE_1"),
            Opcode::ISTORE_2 => write!(f, "ISTORE_2"),
            Opcode::ISTORE_3 => write!(f, "ISTORE_3"),
            Opcode::LSTORE_0 => write!(f, "LSTORE_0"),
            Opcode::LSTORE_1 => write!(f, "LSTORE_1"),
            Opcode::LSTORE_2 => write!(f, "LSTORE_2"),
            Opcode::LSTORE_3 => write!(f, "LSTORE_3"),
            Opcode::FSTORE_0 => write!(f, "FSTORE_0"),
            Opcode::FSTORE_1 => write!(f, "FSTORE_1"),
            Opcode::FSTORE_2 => write!(f, "FSTORE_2"),
            Opcode::FSTORE_3 => write!(f, "FSTORE_3"),
            Opcode::DSTORE_0 => write!(f, "DSTORE_0"),
            Opcode::DSTORE_1 => write!(f, "DSTORE_1"),
            Opcode::DSTORE_2 => write!(f, "DSTORE_2"),
            Opcode::DSTORE_3 => write!(f, "DSTORE_3"),
            Opcode::ASTORE_0 => write!(f, "ASTORE_0"),
            Opcode::ASTORE_1 => write!(f, "ASTORE_1"),
            Opcode::ASTORE_2 => write!(f, "ASTORE_2"),
            Opcode::ASTORE_3 => write!(f, "ASTORE_3"),
            Opcode::IASTORE => write!(f, "IASTORE"),
            Opcode::LASTORE => write!(f, "LASTORE"),
            Opcode::FASTORE => write!(f, "FASTORE"),
            Opcode::DASTORE => write!(f, "DASTORE"),
            Opcode::AASTORE => write!(f, "AASTORE"),
            Opcode::BASTORE => write!(f, "BASTORE"),
            Opcode::CASTORE => write!(f, "CASTORE"),
            Opcode::SASTORE => write!(f, "SASTORE"),

            // stack
            Opcode::POP => write!(f, "POP"),
            Opcode::POP2 => write!(f, "POP2"),
            Opcode::DUP => write!(f, "DUP"),
            Opcode::DUP_X1 => write!(f, "DUP_X1"),
            Opcode::DUP_X2 => write!(f, "DUP_X2"),
            Opcode::DUP2 => write!(f, "DUP2"),
            Opcode::DUP2_X1 => write!(f, "DUP2_X1"),
            Opcode::DUP2_X2 => write!(f, "DUP2_X2"),
            Opcode::SWAP => write!(f, "SWAP"),

            // math
            Opcode::IADD => write!(f, "IADD"),
            Opcode::LADD => write!(f, "LADD"),
            Opcode::FADD => write!(f, "FADD"),
            Opcode::DADD => write!(f, "DADD"),
            Opcode::ISUB => write!(f, "ISUB"),
            Opcode::LSUB => write!(f, "LSUB"),
            Opcode::FSUB => write!(f, "FSUB"),
            Opcode::DSUB => write!(f, "DSUB"),
            Opcode::IMUL => write!(f, "IMUL"),
            Opcode::LMUL => write!(f, "LMUL"),
            Opcode::FMUL => write!(f, "FMUL"),
            Opcode::DMUL => write!(f, "DMUL"),
            Opcode::IDIV => write!(f, "IDIV"),
            Opcode::LDIV => write!(f, "LDIV"),
            Opcode::FDIV => write!(f, "FDIV"),
            Opcode::DDIV => write!(f, "DDIV"),
            Opcode::IREM => write!(f, "IREM"),
            Opcode::LREM => write!(f, "LREM"),
            Opcode::FREM => write!(f, "FREM"),
            Opcode::DREM => write!(f, "DREM"),
            Opcode::INEG => write!(f, "INEG"),
            Opcode::LNEG => write!(f, "LNEG"),
            Opcode::FNEG => write!(f, "FNEG"),
            Opcode::DNEG => write!(f, "DNEG"),
            Opcode::ISHL => write!(f, "ISHL"),
            Opcode::LSHL => write!(f, "LSHL"),
            Opcode::ISHR => write!(f, "ISHR"),
            Opcode::LSHR => write!(f, "LSHR"),
            Opcode::IUSHR => write!(f, "IUSHR"),
            Opcode::LUSHR => write!(f, "LUSHR"),
            Opcode::IAND => write!(f, "IAND"),
            Opcode::LAND => write!(f, "LAND"),
            Opcode::IOR => write!(f, "IOR"),
            Opcode::LOR => write!(f, "LOR"),
            Opcode::IXOR => write!(f, "IXOR"),
            Opcode::LXOR => write!(f, "LXOR"),
            Opcode::IINC => write!(f, "IINC"),

            // comparations
            Opcode::LCMP => write!(f, "LCMP"),
            Opcode::FCMPL => write!(f, "FCMPL"),
            Opcode::FCMPG => write!(f, "FCMPG"),
            Opcode::DCMPL => write!(f, "DCMPL"),
            Opcode::DCMPG => write!(f, "DCMPG"),
            Opcode::IFEQ => write!(f, "IFEQ"),
            Opcode::IFNE => write!(f, "IFNE"),
            Opcode::IFLT => write!(f, "IFLT"),
            Opcode::IFGE => write!(f, "IFGE"),
            Opcode::IFGT => write!(f, "IFGT"),
            Opcode::IFLE => write!(f, "IFLE"),
            Opcode::IF_ICMPEQ => write!(f, "IF_ICMPEQ"),
            Opcode::IF_ICMPNE => write!(f, "IF_ICMPNE"),
            Opcode::IF_ICMPLT => write!(f, "IF_ICMPLT"),
            Opcode::IF_ICMPGE => write!(f, "IF_ICMPGE"),
            Opcode::IF_ICMPGT => write!(f, "IF_ICMPGT"),
            Opcode::IF_ICMPLE => write!(f, "IF_ICMPLE"),
            Opcode::IF_ACMPEQ => write!(f, "IF_ACMPEQ"),
            Opcode::IF_ACMPNE => write!(f, "IF_ACMPNE"),
        }
    }
}
