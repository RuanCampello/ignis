use thiserror::Error;

mod instructions;
mod stack;

#[derive(Error, Debug)]
pub(in crate::vm) enum InterpreterError {
    #[error(transparent)]
    Stack(#[from] stack::StackError),
}
