//! Java byte code instructions definition and processing.

use crate::vm::{Result, interpreter::StackFrames};

mod comparisons;
mod constants;
mod loads;
mod math;
mod stack;
mod stores;

pub(super) mod opcode;

pub(super) fn process(code: u8, classname: &str, frames: &mut StackFrames) -> Result<()> {
    match code {
        0..=20 => constants::process(code, classname, frames),
        21..=53 => loads::process(code, frames),
        54..=86 => stores::process(code, frames),
        87..=95 => stack::process(code, frames),
        96..=132 => math::process(code, frames),
        _ => unreachable!("Tried to process: {code} code"),
    }
}
