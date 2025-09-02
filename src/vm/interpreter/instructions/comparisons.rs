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
        IFEQ => frame.unary_branch(|a| a == 0, opcode),
        IFNE => frame.unary_branch(|a| a != 0, opcode),
        IFLT => frame.unary_branch(|a| a < 0, opcode),
        IFGT => frame.unary_branch(|a| a > 0, opcode),
        IFLE => frame.unary_branch(|a| a <= 0, opcode),
        IFGE => frame.unary_branch(|a| a >= 0, opcode),
        _ => unreachable!("Tried to perform comparation with {code} code"),
    }

    Ok(())
}
