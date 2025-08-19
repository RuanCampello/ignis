//! This module deals with operand stack, local-variables and stack frames.

use crate::vm::VmError;

pub(super) struct StackFrame<V: StackValue> {
    variables: Box<[V]>,
}

pub(super) trait StackValue: Sized {
    /// Retrives the value at `index` from the stack frame.
    fn get(index: usize, frame: &mut StackFrame<Self>) -> Result<(), VmError>;
    /// Set the value at `index` in the stack frame.
    fn set(&self, index: usize, frame: &mut StackFrame<Self>) -> Self;

    fn push(&self, frame: &mut StackFrame<Self>) -> Result<(), VmError>;
    fn pop(frame: &mut StackFrame<Self>) -> Self;
}
