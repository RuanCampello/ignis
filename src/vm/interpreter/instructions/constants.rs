use tracing::trace;

use super::opcode::Opcode::{self, *};
use crate::vm::{
    Result,
    interpreter::stack::{StackError, StackFrames, StackValue},
};

pub(in crate::vm::interpreter::instructions) fn process(
    code: u8,
    classname: &str,
    frames: &mut StackFrames,
) -> Result<()> {
    let frame = frames.last_mut().ok_or(StackError::EmptyStack)?;

    let code = Opcode::from(code);
    match code {
        NOP => {
            frame.next_pc();
            Ok(trace!("NOP"))
        }

        ACONST_NULL => frame.push_const::<i32>(0, code),
        ICONST_0 => frame.push_const::<i32>(0, code),
        ICONST_1 => frame.push_const::<i32>(1, code),
        ICONST_2 => frame.push_const::<i32>(2, code),
        ICONST_3 => frame.push_const::<i32>(3, code),
        ICONST_4 => frame.push_const::<i32>(4, code),
        ICONST_5 => frame.push_const::<i32>(5, code),

        LCONST_0 => frame.push_const::<i64>(0, code),
        LCONST_1 => frame.push_const::<i64>(1, code),

        FCONST_0 => frame.push_const::<f32>(0.0, code),
        FCONST_1 => frame.push_const::<f32>(1.0, code),
        FCONST_2 => frame.push_const::<f32>(2.0, code),

        DCONST_0 => frame.push_const::<f64>(0.0, code),
        DCONST_1 => frame.push_const::<f64>(1.0, code),
        _ => todo!(
            "constant operation not yet handled: {code}",
            code = code as u8
        ),
    }
}
