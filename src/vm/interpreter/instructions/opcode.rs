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
        }
    }
}
