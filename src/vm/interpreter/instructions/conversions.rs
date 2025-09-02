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
        I2L => frame.convert::<i32, i64>(|from| from.into(), opcode),
        I2F => frame.convert::<i32, f32>(|from| from as f32, opcode),
        I2D => frame.convert::<i32, f64>(|from| from.into(), opcode),

        L2I => frame.convert::<i64, i32>(|from| from as _, opcode),
        L2F => frame.convert::<i64, f32>(|from| from as _, opcode),
        L2D => frame.convert::<i64, f64>(|from| from as _, opcode),

        F2I => frame.convert::<f32, i32>(|from| from as _, opcode),
        F2L => frame.convert::<f32, i64>(|from| from as _, opcode),
        F2D => frame.convert::<f32, f64>(|from| from.into(), opcode),

        D2I => frame.convert::<f64, i32>(|from| from as _, opcode),
        D2L => frame.convert::<f64, i64>(|from| from as _, opcode),
        D2F => frame.convert::<f64, f32>(|from| from as _, opcode),

        I2B => frame.convert::<i32, i32>(|from| from as i8 as i32, opcode),
        I2C => frame.convert::<i32, i32>(|from| from as u16 as i32, opcode),
        I2S => frame.convert::<i32, i32>(|from| from as i16 as i32, opcode),
        _ => unreachable!("Tried to call conversion with {code} code"),
    }
}
