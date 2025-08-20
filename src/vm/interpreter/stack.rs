//! This module deals with operand stack, local-variables and stack frames.

use crate::vm::VmError;
use std::{fmt::Display, sync::Arc};
use thiserror::Error;
use tracing::trace;

pub(super) struct StackFrame {
    /// Program counter. This indicates the address of the next bytecode instruction
    /// to be executed.
    pc: usize,
    /// Stores the `pc` before a method invocation. If an exception in thrown during this given
    /// invoked method, this value is restored to the `pc` handle the exception.
    ex_pc: Option<usize>,
    /// Array of local variables for the current method.
    variables: Box<[ValueRef]>,
    /// The operand stack for the current method. It used to store intermediate values
    /// and to pass parameters to and receive results from other methods.
    operand_stack: Stack<ValueRef>,
    /// Shared reference to the bytecode of the method associated with this frame.
    bytecode: Arc<[u8]>,
    current_classname: Arc<str>,
}

pub(super) struct StackFrames {
    frames: Vec<StackFrame>,
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

    #[error("Empty stack frame")]
    EmptyStack,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub(super) enum Value {
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
}

type Result<T> = std::result::Result<T, StackError>;
pub type ValueRef = i32;

pub(super) trait StackValue: Sized + Default + Copy {
    /// Retrives the value at `index` from the stack frame.
    fn get(index: usize, frame: &StackFrame) -> Self;
    /// Set the value at `index` in the stack frame.
    fn set(&self, index: usize, frame: &mut StackFrame);

    /// Push the value onto the operand stack.
    fn push_onto(&self, frame: &mut StackFrame) -> Result<()>;
    /// Pop the value from the operand stack.
    fn pop_from(frame: &mut StackFrame) -> Result<Self>;
}

impl StackFrame {
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
            variables: vec![ValueRef::default(); variables_size].into_boxed_slice(),
            operand_stack: Stack::with_capacity(stack_size),
        }
    }

    pub fn push<V: StackValue>(&mut self, value: V) -> Result<()> {
        value.push_onto(self)
    }

    pub(in crate::vm::interpreter) fn push_const<V: StackValue + Display>(
        &mut self,
        value: V,
        name: &str,
    ) -> Result<()> {
        self.push(value)?;
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

    pub fn pop<V: StackValue>(&mut self) -> Option<V> {
        V::pop_from(self).ok()
    }

    pub fn get_variable(&self, index: usize) -> ValueRef {
        self.variables[index]
    }

    pub fn get<V: StackValue>(&self, index: usize) -> V {
        V::get(index, self)
    }

    pub fn set_variable(&mut self, index: usize, value: ValueRef) {
        self.variables[index] = value;
    }

    pub fn set<V: StackValue>(&mut self, index: usize, value: V) {
        value.set(index, self)
    }

    fn push_ref(&mut self, value: ValueRef) -> Result<()> {
        self.operand_stack.push(value)
    }

    fn pop_ref(&mut self) -> Result<ValueRef> {
        self.operand_stack.pop().ok_or(StackError::EmptyStack)
    }

    fn store_ex_pc(&mut self) {
        self.ex_pc = Some(self.pc);
    }

    fn reset_ex_pc(&mut self) {
        self.ex_pc = None
    }
}

impl StackFrames {
    pub fn add_frame(&mut self, frame: StackFrame) {
        self.frames.push(frame)
    }

    pub fn quit_frame(&mut self) -> Option<StackFrame> {
        let top = self.pop();

        if let Some(next) = self.frames.last_mut() {
            next.reset_ex_pc()
        }

        top
    }

    fn pop(&mut self) -> Option<StackFrame> {
        self.frames.pop()
    }

    pub(super) fn last_mut(&mut self) -> Option<&mut StackFrame> {
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

impl Default for Value {
    fn default() -> Self {
        Value::Int(0)
    }
}

impl StackValue for i32 {
    fn get(index: usize, frame: &StackFrame) -> Self {
        frame.get_variable(index)
    }

    fn set(&self, index: usize, frame: &mut StackFrame) {
        frame.set_variable(index, *self)
    }

    fn push_onto(&self, frame: &mut StackFrame) -> Result<()> {
        frame.push_ref(*self)
    }

    fn pop_from(frame: &mut StackFrame) -> Result<Self> {
        frame.pop_ref()
    }
}

impl StackValue for i64 {
    fn get(index: usize, frame: &StackFrame) -> Self {
        let l = frame.get_variable(index);
        let h = frame.get_variable(index + 1);

        from_i32_to_i64(l, h)
    }

    fn set(&self, index: usize, frame: &mut StackFrame) {
        let l = *self as i32;
        let h = (*self >> 32) as i32;

        frame.set_variable(index, l);
        frame.set_variable(index + 1, h);
    }

    fn push_onto(&self, frame: &mut StackFrame) -> Result<()> {
        let l = *self as i32;
        let h = (*self >> 32) as i32;

        frame.push_ref(l)?;
        frame.push_ref(h)
    }

    fn pop_from(frame: &mut StackFrame) -> Result<Self> {
        let h = frame.pop_ref()?;
        let l = frame.pop_ref()?;

        Ok(from_i32_to_i64(l, h))
    }
}

impl StackValue for f32 {
    fn get(index: usize, frame: &StackFrame) -> Self {
        let v: i32 = frame.get(index);
        f32::from_bits(v as u32)
    }

    fn set(&self, index: usize, frame: &mut StackFrame) {
        frame.set(index, self.to_bits() as i32);
    }

    fn push_onto(&self, frame: &mut StackFrame) -> Result<()> {
        frame.push(self.to_bits() as i32)
    }

    fn pop_from(frame: &mut StackFrame) -> Result<Self> {
        let v: i32 = frame.pop().ok_or(StackError::EmptyStack)?;
        Ok(f32::from_bits(v as u32))
    }
}

impl StackValue for f64 {
    fn get(index: usize, frame: &StackFrame) -> Self {
        let v: i64 = frame.get(index);
        f64::from_bits(v as u64)
    }

    fn set(&self, index: usize, frame: &mut StackFrame) {
        frame.set(index, self.to_bits() as i64);
    }

    fn push_onto(&self, frame: &mut StackFrame) -> Result<()> {
        frame.push(self.to_bits() as i64)
    }

    fn pop_from(frame: &mut StackFrame) -> Result<Self> {
        let v: i64 = frame.pop().ok_or(StackError::EmptyStack)?;
        Ok(f64::from_bits(v as u64))
    }
}

fn from_i32_to_i64(l: i32, h: i32) -> i64 {
    let h = (h as i64) << 32;
    let l = l as u32 as i64;
    h | l
}

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
        let value2 = 19.0f32;
        let value3 = 24.09f32;

        assert!(frame.push(value1).is_ok());
        assert!(frame.push(value2).is_ok());
        assert!(frame.push(value3).is_ok());

        assert_eq!(frame.push(0.0).unwrap_err(), StackError::ExceededStackSize);

        assert_eq!(frame.pop(), Some(value3));
        assert!(frame.push(0.0).is_ok())
    }
}
