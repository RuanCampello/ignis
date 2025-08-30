use crate::vm::{
    Result,
    interpreter::{
        StackFrames,
        instructions::opcode::Opcode::{self, *},
        stack::StackError,
    },
};

pub(in crate::vm::interpreter::instructions) fn process(
    code: u8,
    frames: &mut StackFrames,
) -> Result<()> {
    let frame = frames.last_mut().ok_or(StackError::EmptyStack)?;

    let opcode = Opcode::from(code);
    match opcode {
        ILOAD | ALOAD => frame.positional_load::<i32>(opcode),
        LLOAD => frame.positional_load::<i64>(opcode),
        FLOAD => frame.positional_load::<f32>(opcode),
        DLOAD => frame.positional_load::<f64>(opcode),

        ILOAD_0 | ILOAD_1 | ILOAD_2 | ILOAD_3 => frame.load::<i32, _>(code - ILOAD_0 as u8, opcode),

        LLOAD_0 | LLOAD_1 | LLOAD_2 | LLOAD_3 => frame.load::<i64, _>(code - LLOAD_0 as u8, opcode),

        FLOAD_0 | FLOAD_1 | FLOAD_2 | FLOAD_3 => frame.load::<f32, _>(code - FLOAD_0 as u8, opcode),

        DLOAD_0 | DLOAD_1 | DLOAD_2 | DLOAD_3 => frame.load::<f64, _>(code - DLOAD_0 as u8, opcode),

        ALOAD_0 | ALOAD_1 | ALOAD_2 | ALOAD_3 => frame.load::<i32, _>(code - ALOAD_0 as u8, opcode),

        IALOAD | AALOAD | BALOAD | CALOAD | SALOAD => frame.load_array::<i32>(opcode),
        LALOAD => frame.load_array::<i64>(opcode),
        FALOAD => frame.load_array::<f32>(opcode),
        DALOAD => frame.load_array::<f64>(opcode),

        _ => unreachable!("Tried to load with {code} code"),
    }
}
