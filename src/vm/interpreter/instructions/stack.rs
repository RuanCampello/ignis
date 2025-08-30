use tracing::trace;

use crate::vm::{
    Result,
    interpreter::{
        StackFrames,
        instructions::opcode::Opcode::{self, *},
        stack::{StackError, StackValue},
    },
};

pub(in crate::vm::interpreter::instructions) fn process(
    code: u8,
    frames: &mut StackFrames,
) -> Result<()> {
    let frame = frames.last_mut().ok_or(StackError::EmptyStack)?;

    let opcode = Opcode::from(code);
    match opcode {
        POP => {
            let value: i32 = frame.pop().unwrap();
            frame.next_pc();

            trace!("POP -> {value}");
        }

        POP2 => {
            let value: i32 = frame.pop().unwrap();
            let sec_value: i32 = frame.pop().unwrap();
            frame.next_pc();

            trace!("POP2 -> ({value}, {sec_value})");
        }

        DUP => {
            let value: i32 = frame.pop().unwrap();
            frame.push(value)?;
            frame.push(value)?;

            frame.next_pc();
            trace!("DUP -> {value}");
        }

        DUP_X1 => {
            let value: i32 = frame.pop().unwrap();
            let sec_value: i32 = frame.pop().unwrap();

            frame.push(value)?;
            frame.push(sec_value)?;
            frame.push(value)?;

            frame.next_pc();
            trace!("DUP_X1 -> ({value}, {sec_value})");
        }

        DUP_X2 => {
            let value: i32 = frame.pop().unwrap();
            let sec_value: i32 = frame.pop().unwrap();
            let trd_value: i32 = frame.pop().unwrap();

            frame.push(value)?;
            frame.push(trd_value)?;
            frame.push(sec_value)?;
            frame.push(value)?;

            frame.next_pc();
            trace!("DUP_X2 -> ({value}, {sec_value}, {trd_value})");
        }

        DUP2 => {
            let value: i32 = frame.pop().unwrap();
            let sec_value: i32 = frame.pop().unwrap();

            frame.push(sec_value)?;
            frame.push(value)?;
            frame.push(sec_value)?;
            frame.push(value)?;

            frame.next_pc();
            trace!("DUP2 -> ({value}, {sec_value})");
        }

        DUP2_X1 => {
            let value: i32 = frame.pop().unwrap();
            let sec_value: i32 = frame.pop().unwrap();
            let trd_value: i32 = frame.pop().unwrap();

            frame.push(sec_value)?;
            frame.push(value)?;
            frame.push(trd_value)?;
            frame.push(sec_value)?;
            frame.push(value)?;

            frame.next_pc();
            trace!("DUP2_X1 -> ({value}, {sec_value}, {trd_value})");
        }

        DUP2_X2 => {
            let value: i32 = frame.pop().unwrap();
            let sec_value: i32 = frame.pop().unwrap();
            let trd_value: i32 = frame.pop().unwrap();
            let frth_value: i32 = frame.pop().unwrap();

            frame.push(sec_value)?;
            frame.push(value)?;
            frame.push(frth_value)?;
            frame.push(trd_value)?;
            frame.push(sec_value)?;
            frame.push(value)?;

            frame.next_pc();
            trace!("DUP2_X2 -> ({value}, {sec_value}, {trd_value}, {frth_value})");
        }

        SWAP => {
            let value: i32 = frame.pop().unwrap();
            let sec_value: i32 = frame.pop().unwrap();

            frame.push(value)?;
            frame.push(sec_value)?;

            frame.next_pc();
            trace!("SWAP -> ({value}, {sec_value})");
        }

        _ => unreachable!("Tried to manipulate stack with {code} code"),
    }

    Ok(())
}
