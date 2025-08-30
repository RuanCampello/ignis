use std::ops::Mul;

use crate::vm::{
    Result,
    interpreter::{
        StackFrames,
        instructions::opcode::Opcode::{self, *},
        stack::{StackError, StackValue},
    },
};
use tracing::trace;

const MASK: u32 = 0x3f;

pub(in crate::vm::interpreter::instructions) fn process(
    code: u8,
    frames: &mut StackFrames,
) -> Result<()> {
    let frame = frames.last_mut().ok_or(StackError::EmptyStack)?;

    let opcode = Opcode::from(code);
    match opcode {
        IADD => frame.binary_op(|a: i32, b| a.wrapping_add(b), opcode),
        LADD => frame.binary_op(|a: i64, b| a.wrapping_add(b), opcode),
        FADD => frame.binary_op(|a: f32, b: f32| a + b, opcode),
        DADD => frame.binary_op(|a: f64, b: f64| a + b, opcode),

        ISUB => frame.binary_op(|a: i32, b| a.wrapping_sub(b), opcode),
        LSUB => frame.binary_op(|a: i64, b| a.wrapping_sub(b), opcode),
        FSUB => frame.binary_op(|a: f32, b: f32| a - b, opcode),
        DSUB => frame.binary_op(|a: f64, b: f64| a - b, opcode),

        IMUL => frame.binary_op(|a: i32, b: i32| a.wrapping_mul(b), opcode),
        LMUL => frame.binary_op(|a: i64, b: i64| a.wrapping_mul(b), opcode),
        FMUL => frame.binary_op(|a: f32, b: f32| a.mul(b), opcode),
        DMUL => frame.binary_op(|a: f64, b: f64| a.mul(b), opcode),

        IDIV => frame.binary_op(|a: i32, b| a.wrapping_div(b), opcode),
        LDIV => frame.binary_op(|a: i64, b| a.wrapping_div(b), opcode),
        FDIV => frame.binary_op(|a: f32, b: f32| a / b, opcode),
        DDIV => frame.binary_op(|a: f64, b: f64| a / b, opcode),

        IREM => frame.binary_op(|a: i32, b| a.wrapping_rem(b), opcode),
        LREM => frame.binary_op(|a: i64, b| a.wrapping_rem(b), opcode),
        FREM => frame.binary_op(|a: f32, b: f32| a % b, opcode),
        DREM => frame.binary_op(|a: f64, b: f64| a % b, opcode),

        LSHL => frame.binary_op(|a: i64, b: i32| a << (b as u32 & MASK), opcode),
        LSHR => frame.binary_op(|a: i64, b: i32| a >> (b as u32 & MASK), opcode),
        ISHR => frame.binary_op(|a: i32, b: i32| a >> (b as u32 & MASK), opcode),
        IUSHR => frame.binary_op(
            |a: i32, b: i32| (a as u32 >> (b as u32 & MASK)) as i32,
            opcode,
        ),
        LUSHR => frame.binary_op(
            |a: i64, b: i32| (a as u64 >> (b as u32 & MASK)) as i64,
            opcode,
        ),

        IAND => frame.binary_op(|a: i32, b: i32| a & b, opcode),
        LAND => frame.binary_op(|a: i64, b: i64| a & b, opcode),
        IOR => frame.binary_op(|a: i32, b: i32| a | b, opcode),
        LOR => frame.binary_op(|a: i64, b: i64| a | b, opcode),
        IXOR => frame.binary_op(|a: i32, b: i32| a ^ b, opcode),
        LXOR => frame.binary_op(|a: i64, b: i64| a ^ b, opcode),

        IINC => frame.increment(
            |f| f.get_next_byte() as usize,
            |f| f.get_next_byte() as i8 as i32,
            opcode,
        ),
        _ => unreachable!("Tried perform math operation with {code} code"),
    }
}
