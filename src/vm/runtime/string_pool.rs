//! Interning pool for `java/lang/String` instances
//!
//! The JVM guarantees that a given string literal resolves to a single shared
//! `java/lang/String` object. The interning cache itself lives in the
//! [HEAP](crate::vm::runtime::heap).

use crate::vm::{
    self, Result, interpreter::executor::Executor, interpreter::stack::Value,
    runtime::RuntimeError, runtime::heap::HEAP,
};

const STRING: &str = "java/lang/String";

/// Encoding marker a `java/lang/String` stores alongside its `byte[] value`
enum Coder {
    /// single-byte [ISO-8859-1](https://en.wikipedia.org/wiki/ISO/IEC_8859-1)
    Latin1,
    /// two-byte UTF-16
    Utf16,
}

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

/// allocates a `java/lang/String[]` on the heap, interning each entry and return is ref
pub(in crate::vm) fn create_string_array(properties: &[&str]) -> Result<i32> {
    let class = format!("[L{STRING};");
    let array_ref = HEAP.allocate_array(&class, properties.len() as i32);

    for (index, property) in properties.iter().enumerate() {
        let string_ref = get(property)?;
        HEAP.set_array_value(array_ref, index as i32, vec![string_ref])?;
    }

    Ok(array_ref)
}

/// decodes the interned `java/lang/String` at `string` back into a rust string,
/// reading its `byte[] value` and `coder` exactly as the jdk packed them
pub(in crate::vm) fn get_by_ref(string: i32) -> Result<String> {
    let value = HEAP.get_field_value(string, STRING, "value")?[0];
    let coder = HEAP.get_field_value(string, STRING, "coder")?[0];
    let bytes = HEAP.array_bytes(value)?;

    match Coder::try_from(coder)? {
        Coder::Latin1 => Ok(bytes.into_iter().map(|byte| byte as char).collect()),
        Coder::Utf16 => {
            let units = bytes
                .chunks_exact(2)
                .map(|pair| u16::from_ne_bytes([pair[0], pair[1]]))
                .collect::<Vec<_>>();

            String::from_utf16(&units).map_err(|e| RuntimeError::Execution(e.to_string()).into())
        }
    }
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

impl TryFrom<i32> for Coder {
    type Error = vm::VmError;

    fn try_from(value: i32) -> Result<Self> {
        match value {
            0 => Ok(Self::Latin1),
            1 => Ok(Self::Utf16),
            other => Err(RuntimeError::Execution(format!("unknown string coder: {other}")).into()),
        }
    }
}
