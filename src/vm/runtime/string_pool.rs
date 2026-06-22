//! Interning pool for `java/lang/String` instances
//!
//! The JVM guarantees that a given string literal resolves to a single shared
//! `java/lang/String` object. The interning cache itself lives in the
//! [HEAP](crate::vm::runtime::heap).

use crate::vm::{
    Result, interpreter::executor::Executor, interpreter::stack::Value, runtime::heap::HEAP,
};

const STRING: &str = "java/lang/String";

/// Heap reference of the interned `java/lang/String` for `value`, creating and
/// caching it on first use
pub(in crate::vm) fn get(value: &str) -> Result<i32> {
    if let Some(reference) = HEAP.interned_string(value) {
        return Ok(reference);
    }

    let reference = create(value)?;
    HEAP.intern_string(value, reference);

    Ok(reference)
}

fn create(value: &str) -> Result<i32> {
    if value.is_empty() {
        return create_empty();
    }

    let codepoints = value.chars().map(|c| c as i32).collect::<Vec<_>>();
    let array = HEAP.allocate_int_array(&codepoints);

    // String(int[] codePoints, int offset, int count)
    let args = [
        Value::from(array),
        Value::from(0),
        Value::from(codepoints.len() as i32),
    ];

    Executor::constructor(STRING, "<init>:([III)V", &args)
}

fn create_empty() -> Result<i32> {
    let array = HEAP.allocate_array_with_values("[B", Vec::new());

    // String(byte[] value, byte coder), where `0` is the LATIN1 coder
    let args = [Value::from(array), Value::from(0)];

    Executor::constructor(STRING, "<init>:([BB)V", &args)
}
