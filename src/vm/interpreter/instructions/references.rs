use super::opcode::Opcode::{self, *};
use crate::vm::{
    Result,
    descriptor::MethodDescriptor,
    interpreter::{
        native,
        stack::{StackError, StackFrames, ValueRef},
    },
    method_area::class::CLASSES,
    runtime::RuntimeError,
};

pub(in crate::vm::interpreter::instructions) fn process(
    code: u8,
    classname: &str,
    frames: &mut StackFrames,
) -> Result<()> {
    let code = Opcode::from(code);
    match code {
        INVOKE_STATIC => invoke_static(classname, frames),
        _ => todo!("reference opcode not yet handled: {code}"),
    }
}

fn invoke_static(classname: &str, frames: &mut StackFrames) -> Result<()> {
    let index = read_index(frames)?;

    let class = CLASSES.get(classname)?;
    let (owner, name, descriptor) = class
        .constant_pool
        .member_ref(index)
        .map_err(|e| RuntimeError::Execution(e.to_string()))?;

    let signature = format!("{name}:{descriptor}");
    let argument_size = descriptor
        .parse::<MethodDescriptor>()
        .map_err(|e| RuntimeError::Execution(e.to_string()))?
        .arguments_size();
    let args = pop_args(frames, argument_size)?;

    let method = CLASSES.get(owner)?.get_method(&signature)?;

    if method.is_native() {
        let result = native::invoke(owner, &signature, &args)?;
        let caller = frames.last_mut().ok_or(StackError::EmptyStack)?;
        for slot in result.into_iter().rev() {
            caller.push(slot)?;
        }
    } else {
        let mut frame = method.new_frame()?;
        for (local, value) in args.into_iter().enumerate() {
            frame.set_variable(local, value);
        }
        frames.add_frame(frame);
    }

    Ok(())
}

fn read_index(frames: &mut StackFrames) -> Result<u16> {
    let frame = frames.last_mut().ok_or(StackError::EmptyStack)?;
    let high = frame.get_next_byte() as u16;
    let low = frame.get_next_byte() as u16;
    frame.next_pc();

    Ok((high << 8) | low)
}

fn pop_args(frames: &mut StackFrames, count: usize) -> Result<Vec<ValueRef>> {
    let frame = frames.last_mut().ok_or(StackError::EmptyStack)?;
    let mut args = Vec::with_capacity(count);
    for _ in 0..count {
        args.push(frame.pop::<ValueRef>().ok_or(StackError::StackUnderflow)?);
    }
    args.reverse();

    Ok(args)
}
