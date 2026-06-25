//! Java byte code instructions definition and processing

use crate::vm::{
    Result,
    interpreter::stack::{StackFrames, ValueRef},
};

mod comparisons;
mod constants;
mod control;
mod conversions;
mod extended;
mod loads;
mod math;
mod references;
mod stack;
mod stores;

pub(in crate::vm::interpreter) mod opcode;

pub(in crate::vm::interpreter) fn process(
    code: u8,
    classname: &str,
    frames: &mut StackFrames,
) -> Result<Vec<ValueRef>> {
    if let 167..=177 = code {
        return control::process(code, frames);
    }

    match code {
        0..=20 => constants::process(code, classname, frames),
        21..=53 => loads::process(code, frames),
        54..=86 => stores::process(code, frames),
        87..=95 => stack::process(code, frames),
        96..=132 => math::process(code, frames),
        133..=147 => conversions::process(code, frames),
        148..=166 => comparisons::process(code, frames),
        178..=195 => references::process(code, classname, frames),
        196..=200 => extended::process(code, frames),
        _ => unreachable!("Tried to process: {code} code"),
    }?;

    Ok(vec![])
}
