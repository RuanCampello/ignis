use crate::vm::{
    Result,
    interpreter::{
        StackFrames,
        instructions::opcode::{Opcode, Opcode::*},
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
        ISTORE | ASTORE => frame.positional_store::<i32>(opcode),
        LSTORE => frame.positional_store::<i64>(opcode),
        FSTORE => frame.positional_store::<f32>(opcode),
        DSTORE => frame.positional_store::<f64>(opcode),

        ISTORE_0 | ISTORE_1 | ISTORE_2 | ISTORE_3 => {
            frame.store::<i32, _>(code - ISTORE_0 as u8, opcode)
        }

        LSTORE_0 | LSTORE_1 | LSTORE_2 | LSTORE_3 => {
            frame.store::<i64, _>(code - LSTORE_0 as u8, opcode)
        }

        FSTORE_0 | FSTORE_1 | FSTORE_2 | FSTORE_3 => {
            frame.store::<f32, _>(code - FSTORE_0 as u8, opcode)
        }

        DSTORE_0 | DSTORE_1 | DSTORE_2 | DSTORE_3 => {
            frame.store::<f64, _>(code - DSTORE_0 as u8, opcode)
        }

        ASTORE_0 | ASTORE_1 | ASTORE_2 | ASTORE_3 => {
            frame.store::<i32, _>(code - ASTORE_0 as u8, opcode)
        }

        IALOAD | AASTORE | BASTORE | CASTORE | SASTORE => frame.store_array::<i32>(opcode),
        LASTORE => frame.store_array::<i64>(opcode),
        FASTORE => frame.store_array::<f32>(opcode),
        DASTORE => frame.store_array::<f64>(opcode),

        _ => unreachable!("Tried to store {code} code"),
    }
}
