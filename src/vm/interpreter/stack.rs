//! This module deals with operand stack, local-variables and stack frames.

use crate::vm::VmError;
use std::{fmt::Display, sync::Arc};
use thiserror::Error;
use tracing::trace;

pub(super) struct StackFrame<V: StackValue> {
    /// Program counter. This indicates the address of the next bytecode instruction
    /// to be executed.
    pc: usize,
    /// Stores the `pc` before a method invocation. If an exception in thrown during this given
    /// invoked method, this value is restored to the `pc` handle the exception.
    ex_pc: Option<usize>,
    /// Array of local variables for the current method.
    variables: Box<[V]>,
    /// The operand stack for the current method. It used to store intermediate values
    /// and to pass parameters to and receive results from other methods.
    operand_stack: Stack<V>,
    /// Shared reference to the bytecode of the method associated with this frame.
    bytecode: Arc<[u8]>,
    current_classname: Arc<str>,
}

pub(super) struct StackFrames<V: StackValue> {
    frames: Vec<StackFrame<V>>,
}

pub(super) struct Stack<T> {
    capacity: usize,
    inner: Vec<T>,
}

#[derive(Error, Debug, PartialEq)]
pub(in crate::vm) enum StackError {
    #[error("Exceeded max stack size")]
    ExceededStackSize,

    #[error("Operand stack underflow")]
    StackUnderflow,
}

type Result<T> = std::result::Result<T, StackError>;

pub(super) trait StackValue: Sized + Default + Copy {
    /// Retrives the value at `index` from the stack frame.
    fn get(index: usize, frame: &StackFrame<Self>) -> Self;
    /// Set the value at `index` in the stack frame.
    fn set(&self, index: usize, frame: &mut StackFrame<Self>);

    /// Push the value onto the operand stack.
    fn push(&self, frame: &mut StackFrame<Self>) -> Result<()>;
    /// Pop the value from the operand stack.
    fn pop(frame: &mut StackFrame<Self>) -> Result<Self>;
}

impl<V: StackValue> StackFrame<V> {
    pub fn new(
        variables_size: usize,
        stack_size: usize,
        bytecode: Arc<[u8]>,
        current_classname: Arc<str>,
    ) -> Self {
        Self {
            bytecode,
            current_classname,
            pc: 0,
            ex_pc: None,
            variables: vec![V::default(); variables_size].into_boxed_slice(),
            operand_stack: Stack::with_capacity(stack_size),
        }
    }

    pub fn push(&mut self, value: V) -> Result<()> {
        value.push(self)
    }

    pub(in crate::vm::interpreter) fn push_const<T>(&mut self, value: T, name: &str) -> Result<()>
    where
        V: StackValue + From<T>,
        T: StackValue + Display,
    {
        self.push(value.into())?;
        self.next_pc();

        trace!("{name} -> {value}");

        Ok(())
    }

    pub fn next_pc(&mut self) {
        self.step_pc(1);
    }

    pub fn step_pc(&mut self, step: i16) {
        match step >= 0 {
            true => self.pc += step as usize,
            false => self.pc -= (-step) as usize,
        }
    }

    pub fn pop(&mut self) -> Option<V> {
        V::pop(self).ok()
    }

    pub fn get_variable(&self, index: usize) -> V {
        self.variables[index]
    }

    pub fn set_variable(&mut self, index: usize, value: V) {
        self.variables[index] = value;
    }

    fn store_ex_pc(&mut self) {
        self.ex_pc = Some(self.pc);
    }

    fn reset_ex_pc(&mut self) {
        self.ex_pc = None
    }
}

impl<V: StackValue> StackFrames<V> {
    pub fn add_frame(&mut self, frame: StackFrame<V>) {
        self.frames.push(frame)
    }

    pub fn quit_frame(&mut self) -> Option<StackFrame<V>> {
        let top = self.pop();

        if let Some(next) = self.frames.last_mut() {
            next.reset_ex_pc()
        }

        top
    }

    fn pop(&mut self) -> Option<StackFrame<V>> {
        self.frames.pop()
    }

    fn last_mut(&mut self) -> Option<&mut StackFrame<V>> {
        self.frames.last_mut()
    }
}

impl<T> Stack<T> {
    fn with_capacity(capacity: usize) -> Self {
        Self {
            capacity,
            inner: Vec::with_capacity(capacity),
        }
    }

    fn push(&mut self, value: T) -> Result<()> {
        if self.capacity <= self.inner.len() {
            return Err(StackError::ExceededStackSize);
        }

        Ok(self.inner.push(value))
    }

    fn pop(&mut self) -> Option<T> {
        self.inner.pop()
    }

    fn clear(&mut self) {
        self.inner.clear();
    }
}

macro_rules! stack_value {
    ($t:ty) => {
        impl StackValue for $t {
            fn get(index: usize, frame: &StackFrame<Self>) -> Self {
                frame.variables[index]
            }
            fn set(&self, index: usize, frame: &mut StackFrame<Self>) {
                frame.variables[index] = *self;
            }
            fn push(&self, frame: &mut StackFrame<Self>) -> Result<()> {
                frame.operand_stack.push(*self)
            }
            fn pop(frame: &mut StackFrame<Self>) -> Result<Self> {
                frame.operand_stack.pop().ok_or(StackError::StackUnderflow)
            }
        }
    };
}

stack_value!(i32);
stack_value!(i64);
stack_value!(f32);
stack_value!(f64);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frame_stack_basics() {
        let mut frame = StackFrame::new(10, 5, Arc::default(), Arc::default());

        let value1 = 10;
        let value2 = 20;

        assert!(frame.push(value1).is_ok());
        assert!(frame.push(value2).is_ok());

        assert_eq!(frame.pop(), Some(value2));
        assert_eq!(frame.pop(), Some(value1));
    }

    #[test]
    fn frame_stack_overflow() {
        let mut frame = StackFrame::new(5, 3, Arc::default(), Arc::default());

        let value1 = 15.12f32;
        let value2 = 19.0;
        let value3 = 24.09;

        assert!(frame.push(value1).is_ok());
        assert!(frame.push(value2).is_ok());
        assert!(frame.push(value3).is_ok());

        assert_eq!(frame.push(0.0).unwrap_err(), StackError::ExceededStackSize);

        assert_eq!(frame.pop(), Some(value3));
        assert!(frame.push(0.0).is_ok())
    }
}
