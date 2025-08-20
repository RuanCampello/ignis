use tracing::trace;

use super::opcode::Opcode::{self, *};
use crate::vm::interpreter::stack::{StackError, StackFrames, StackValue};

type Result<T> = std::result::Result<T, StackError>;

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

        ACONST_NULL => frame.push_const::<i32>(0, "ACONST_NULL"),
        ICONST_0 => frame.push_const::<i32>(0, "ICONST_0"),
        ICONST_1 => frame.push_const::<i32>(1, "ICONST_1"),
        ICONST_2 => frame.push_const::<i32>(2, "ICONST_2"),
        ICONST_3 => frame.push_const::<i32>(3, "ICONST_3"),
        ICONST_4 => frame.push_const::<i32>(4, "ICONST_4"),
        ICONST_5 => frame.push_const::<i32>(5, "ICONST_5"),

        LCONST_0 => frame.push_const::<i64>(0, "LCONST_0"),
        LCONST_1 => frame.push_const::<i64>(1, "LCONST_1"),

        FCONST_0 => frame.push_const::<f32>(0.0, "FCONST_0"),
        FCONST_1 => frame.push_const::<f32>(1.0, "FCONST_1"),
        FCONST_2 => frame.push_const::<f32>(2.0, "FCONST_2"),

        DCONST_0 => frame.push_const::<f64>(0.0, "DCONST_0"),
        DCONST_1 => frame.push_const::<f64>(1.0, "DCONST_1"),
        _ => todo!(
            "constant operation not yet handled: {code}",
            code = code as u8
        ),
    }
}
