use thiserror::Error;

use crate::vm::{
    Result, VmError,
    interpreter::stack::{StackError, StackFrames, ValueRef},
};

pub(in crate::vm) use stack::StackFrame;

mod executor;
mod instructions;
mod stack;

#[derive(Error, Debug)]
pub(in crate::vm) enum InterpreterError {
    #[error(transparent)]
    Stack(#[from] stack::StackError),
}

pub(in crate::vm::interpreter) fn execute(frame: StackFrame) -> Result<Vec<ValueRef>> {
    let mut frames = StackFrames::from(vec![frame]);
    let mut last = vec![];

    while !frames.is_empty() {
        let (classname, code, pc) = {
            let frame = frames.last().ok_or(StackError::EmptyStack)?;

            (
                frame.current_classname.to_string(),
                frame.current_byte(),
                frame.pc,
            )
        };

        instructions::process(code, &classname, &mut frames)?
    }

    Ok(last)
}

impl From<StackError> for VmError {
    fn from(value: StackError) -> Self {
        Self::Interpreter(InterpreterError::Stack(value))
    }
}
