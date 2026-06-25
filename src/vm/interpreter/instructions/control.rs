use super::opcode::Opcode::{self, *};
use crate::vm::{
    Result,
    interpreter::stack::{StackError, StackFrames, StackValue, ValueRef},
};

pub(in crate::vm::interpreter::instructions) fn process(
    code: u8,
    frames: &mut StackFrames,
) -> Result<Vec<ValueRef>> {
    let code = Opcode::from(code);
    match code {
        I_RETURN | A_RETURN => perform_return::<i32>(frames),
        L_RETURN => perform_return::<i64>(frames),
        F_RETURN => perform_return::<f32>(frames),
        D_RETURN => perform_return::<f64>(frames),
        RETURN => {
            frames.quit_frame();
            Ok(vec![])
        }
        GOTO => {
            frames
                .last_mut()
                .ok_or(StackError::EmptyStack)?
                .branch(GOTO);
            Ok(vec![])
        }
        _ => todo!("control opcode not yet handled: {code}"),
    }
}

fn perform_return<V: StackValue>(frames: &mut StackFrames) -> Result<Vec<ValueRef>> {
    let value: V = {
        let frame = frames.last_mut().ok_or(StackError::EmptyStack)?;
        frame.pop::<V>().ok_or(StackError::StackUnderflow)?
    };

    frames.quit_frame();
    if let Some(caller) = frames.last_mut() {
        caller.push(value)?;
    }

    Ok(value.to_vec())
}
