//! Java byte code instructions definition and processing.

use crate::vm::{Result, interpreter::StackFrames};

mod constants;
mod loads;
pub(super) mod opcode;

pub(super) fn process(code: u8, classname: &str, frames: &mut StackFrames) -> Result<()> {
    match code {
        0..=20 => constants::process(code, classname, frames),
        21..=53 => loads::process(code, frames),
        _ => unreachable!("Tried to process: {code} code"),
    }
}
