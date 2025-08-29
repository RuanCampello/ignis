//! Java byte code instructions definition and processing.

use crate::vm::interpreter::{StackFrames, stack::Result};

mod constants;
mod opcode;

pub(super) fn process(code: u8, classname: &str, frames: &mut StackFrames) -> Result<()> {
    match code {
        0..20 => constants::process(code, classname, frames),
        _ => unreachable!("Tried to process: {code} code"),
    }
}
