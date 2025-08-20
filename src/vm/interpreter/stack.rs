//! This module deals with operand stack, local-variables and stack frames.

use crate::vm::VmError;
use thiserror::Error;

pub(super) struct StackFrame<V: StackValue> {
    pc: usize,
    variables: Box<[V]>,
    operand_stack: Stack<V>,
}

pub(super) struct Stack<T> {
    capacity: usize,
    inner: Vec<T>,
}

#[derive(Error, Debug)]
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
    pub fn new(variables_size: usize, stack_size: usize) -> Self {
        Self {
            pc: 0,
            variables: vec![V::default(); variables_size].into_boxed_slice(),
            operand_stack: Stack::with_capacity(stack_size),
        }
    }

    pub fn get_variable(&self, index: usize) -> V {
        self.variables[index]
    }

    pub fn set_variable(&mut self, index: usize, value: V) {
        self.variables[index] = value;
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
