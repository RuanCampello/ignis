use crate::vm::{
    Result,
    interpreter::{
        StackFrames,
        instructions::opcode::Opcode::{self, *},
        stack::StackError,
    },
};

pub(in crate::vm::interpreter::instructions) fn process(
    code: u8,
    frames: &mut StackFrames,
) -> Result<()> {
    let frame = frames.last_mut().ok_or(StackError::EmptyStack)?;

    let opcode = Opcode::from(code);
    match opcode {
        LCMP => frame.compare::<i64>(0, opcode)?,
        FCMPL => frame.compare::<f32>(-1, opcode)?,
        DCMPL => frame.compare::<f32>(-1, opcode)?,
        DCMPG => frame.compare::<f64>(1, opcode)?,
        FCMPG => frame.compare::<f32>(1, opcode)?,

        IFEQ => frame.unary_branch(|a| a == 0, opcode),
        IFNE => frame.unary_branch(|a| a != 0, opcode),
        IFLT => frame.unary_branch(|a| a < 0, opcode),
        IFGT => frame.unary_branch(|a| a > 0, opcode),
        IFLE => frame.unary_branch(|a| a <= 0, opcode),
        IFGE => frame.unary_branch(|a| a >= 0, opcode),

        IF_ICMPEQ | IF_ACMPEQ => frame.binary_branch(|a, b| a == b, opcode),
        IF_ICMPNE | IF_ACMPNE => frame.binary_branch(|a, b| a != b, opcode),
        IF_ICMPLT => frame.binary_branch(|a, b| a < b, opcode),
        IF_ICMPLE => frame.binary_branch(|a, b| a <= b, opcode),
        IF_ICMPGT => frame.binary_branch(|a, b| a > b, opcode),
        IF_ICMPGE => frame.binary_branch(|a, b| a >= b, opcode),

        _ => unreachable!("Tried to perform comparation with {code} code"),
    }

    Ok(())
}
