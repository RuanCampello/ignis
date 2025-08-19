//! Java bytecode opcode [instructions](https://docs.oracle.com/javase/specs/jvms/se24/html/jvms-6.html).

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[repr(u8)]
#[allow(non_camel_case_types)]

pub(crate) enum Opcode {
    // constants-related instructions
    /// Do nothing; execution proceeds to the next instruction.
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
