use thiserror::Error;

use crate::vm::{
    Result, VmError,
    interpreter::stack::{StackError, StackFrames, ValueRef},
};

pub(in crate::vm) use stack::StackFrame;

pub mod executor;
mod instructions;
pub(in crate::vm) mod ldc;
pub(in crate::vm::interpreter) mod native;
pub(in crate::vm) mod stack;
pub mod static_method;

/// errors from the execution phase: running bytecode in a method frame
///
/// the counterpart to [`RuntimeError`](crate::vm::runtime::RuntimeError) under
/// [`VmError`](crate::vm::VmError), scoped to what goes wrong *while interpreting*
/// opcodes rather than while loading or linking classes
#[derive(Error, Debug)]
pub enum InterpreterError {
    /// operand stack / frame fault (overflow, underflow, empty frame)
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

        last = instructions::process(code, &classname, &mut frames)?;
    }

    Ok(last)
}

impl From<StackError> for VmError {
    fn from(value: StackError) -> Self {
        Self::Interpreter(InterpreterError::Stack(value))
    }
}
