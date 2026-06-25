use super::opcode::Opcode::{self, *};
use crate::vm::{
    Result,
    interpreter::stack::{StackError, StackFrames},
};

pub(in crate::vm::interpreter::instructions) fn process(
    code: u8,
    frames: &mut StackFrames,
) -> Result<()> {
    let frame = frames.last_mut().ok_or(StackError::EmptyStack)?;

    match Opcode::from(code) {
        IF_NULL => frame.unary_branch(|reference| reference == 0, IF_NULL),
        IF_NON_NULL => frame.unary_branch(|reference| reference != 0, IF_NON_NULL),
        opcode => todo!("extended opcode not yet handled: {opcode} (raw {code})"),
    }

    Ok(())
}
