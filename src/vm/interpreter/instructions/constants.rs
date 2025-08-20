use super::opcode::Opcode::*;
use crate::vm::interpreter::{
    InterpreterError,
    stack::{StackError, StackFrames, StackValue},
};

type Result<T> = std::result::Result<T, InterpreterError>;

pub(in crate::vm::interpreter::instructions) fn process<V: StackValue>(
    code: u8,
    classname: &str,
    frames: &mut StackFrames,
) -> Result<()> {
    let frame = frames
        .last_mut()
        .ok_or(InterpreterError::Stack(StackError::EmptyStack))?;

    todo!()
}
