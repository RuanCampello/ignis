use crate::vm::interpreter::{
    StackFrames,
    instructions::opcode::{Opcode, Opcode::*},
    stack::{Result, StackError},
};

pub(in crate::vm::interpreter::instructions) fn process(
    code: u8,
    frames: &mut StackFrames,
) -> Result<()> {
    let frame = frames.last_mut().ok_or(StackError::EmptyStack)?;

    let opcode = Opcode::from(code);
    match opcode {
        ILOAD => frame.positional_load::<i32>(opcode)?,
        ALOAD => frame.positional_load::<i32>(opcode)?,
        LLOAD => frame.positional_load::<i64>(opcode)?,
        FLOAD => frame.positional_load::<f32>(opcode)?,
        DLOAD => frame.positional_load::<f64>(opcode)?,

        ILOAD_0 | ILOAD_1 | ILOAD_2 | ILOAD_3 => {
            frame.load::<i32, _>(code - ILOAD_0 as u8, opcode)?
        }

        _ => unreachable!("Tried to load with {code} code"),
    }
    todo!()
}
