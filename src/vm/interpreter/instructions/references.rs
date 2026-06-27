use super::opcode::Opcode::{self, *};
use crate::vm::{
    Result,
    descriptor::{MethodDescriptor, TypeDescriptor},
    interpreter::{
        native,
        stack::{StackError, StackFrames, ValueRef},
        static_method::Static,
    },
    method_area::class::CLASSES,
    runtime::RuntimeError,
};

pub(in crate::vm::interpreter::instructions) fn process(
    code: u8,
    classname: &str,
    frames: &mut StackFrames,
) -> Result<()> {
    match Opcode::from(code) {
        GET_STATIC => {
            let index = read_index(frames)?;

            let class = CLASSES.get(classname)?;
            let (owner, name, _) = class
                .constant_pool
                .member_ref(index)
                .map_err(|e| RuntimeError::Execution(e.to_string()))?;

            Static::initialise(owner)?;

            let field = CLASSES.get(owner)?.get_static(name).ok_or_else(|| {
                RuntimeError::Execution(format!("static field {owner}.{name} not found"))
            })?;
            let value = field.value()?;

            let frame = frames.last_mut().ok_or(StackError::EmptyStack)?;
            for slot in value.into_iter().rev() {
                frame.push(slot)?;
            }

            Ok(())
        }

        PUT_STATIC => {
            let index = read_index(frames)?;

            let class = CLASSES.get(classname)?;
            let (owner, name, descriptor) = class
                .constant_pool
                .member_ref(index)
                .map_err(|e| RuntimeError::Execution(e.to_string()))?;
            let slots = descriptor
                .parse::<TypeDescriptor>()
                .map_err(|e| RuntimeError::Execution(e.to_string()))?
                .size();

            Static::initialise(owner)?;

            let mut value = Vec::with_capacity(slots);
            let frame = frames.last_mut().ok_or(StackError::EmptyStack)?;
            for _ in 0..slots {
                value.push(frame.pop::<ValueRef>().ok_or(StackError::StackUnderflow)?);
            }

            CLASSES
                .get(owner)?
                .get_static(name)
                .ok_or_else(|| {
                    RuntimeError::Execution(format!("static field {owner}.{name} not found"))
                })?
                .set(value)?;

            Ok(())
        }

        INVOKE_STATIC => {
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

            match method.is_native() {
                true => {
                    let result = native::invoke(owner, &signature, &args, frames)?;
                    let caller = frames.last_mut().ok_or(StackError::EmptyStack)?;
                    for slot in result.into_iter().rev() {
                        caller.push(slot)?;
                    }
                }
                _ => {
                    let mut frame = method.new_frame()?;
                    for (local, value) in args.into_iter().enumerate() {
                        frame.set_variable(local, value);
                    }
                    frames.add_frame(frame);
                }
            };

            Ok(())
        }

        INVOKE_SPECIAL => todo!(),
        opcode => todo!("reference opcode not yet handled: {opcode} (raw {code})"),
    }
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
