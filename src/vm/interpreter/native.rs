//! Dispatch for `native` methods implemented in Rust rather than Java bytecode.

use crate::vm::{
    Result,
    interpreter::stack::{StackFrames, ValueRef},
    runtime::RuntimeError,
};
use once_cell::sync::Lazy;
use std::collections::HashMap;

type BasicNative = fn(&[ValueRef]) -> Result<Vec<ValueRef>>;
type WithFramesNative = fn(&[ValueRef], &mut StackFrames) -> Result<Vec<ValueRef>>;

/// A native method implementation, parameterised by what it needs access to
enum NativeMethod {
    /// operates purely on its arguments
    Basic(BasicNative),
    /// also needs the call stack
    WithFrames(WithFramesNative),
}

static NATIVE_TABLE: Lazy<HashMap<&str, NativeMethod>> = Lazy::new(|| {
    HashMap::from([(
        "java/lang/Object:registerNatives:()V",
        NativeMethod::Basic(void),
    )])
});

/// invokes the native method `class.signature` with `args`, returning its result slots.
pub(in crate::vm::interpreter) fn invoke(
    class: &str,
    signature: &str,
    args: &[ValueRef],
    frames: &mut StackFrames,
) -> Result<Vec<ValueRef>> {
    let key = format!("{class}:{signature}");

    match NATIVE_TABLE.get(key.as_str()) {
        Some(NativeMethod::Basic(native)) => native(args),
        Some(NativeMethod::WithFrames(native)) => native(args, frames),
        // every class calls `registerNatives` during its static init, so we treat it as a noop
        None if signature.starts_with("registerNatives:") => Ok(vec![]),
        None => Err(RuntimeError::Execution(format!(
            "native method not implemented: {key} ({args:?})"
        ))
        .into()),
    }
}

#[inline]
const fn void(_: &[ValueRef]) -> Result<Vec<ValueRef>> {
    Ok(vec![])
}
