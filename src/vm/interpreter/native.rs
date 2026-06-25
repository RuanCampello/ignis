//! Dispatch for `native` methods implemented in Rust rather than Java bytecode

use crate::vm::{Result, interpreter::stack::ValueRef, runtime::RuntimeError};

/// invokes the native method `class.signature` with `args`, returning its result slots.
pub(in crate::vm::interpreter) fn invoke(
    class: &str,
    signature: &str,
    args: &[ValueRef],
) -> Result<Vec<ValueRef>> {
    let (name, _descriptor) = signature.split_once(':').unwrap_or((signature, ""));

    match name {
        "registerNatives" => Ok(vec![]),
        _ => Err(RuntimeError::Execution(format!(
            "native method not implemented: {class}.{signature} ({args:?})"
        ))
        .into()),
    }
}
