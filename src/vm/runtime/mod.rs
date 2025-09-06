//! This module defines the JVM runtime data areas, including the heap, method area, and related
//! resources. It holds the live objects, class metadata, and constant pools needed during execution,
//! providing the dynamic state that the VM operates on.

use thiserror::Error;
pub(in crate::vm) mod heap;
pub(in crate::vm) mod method_area;

#[derive(Error, Debug)]
pub(in crate::vm) enum RuntimeError {
    #[error("METHOD_AREA was already initialised")]
    MethodAreaInitialised,

    #[error("Method with signature {0} does not exists")]
    MethodNotFound(String),

    #[error("Attempted to access non-existing field: '{field}' of object of class '{classname}'")]
    InvalidObjectAcess { classname: String, field: String },

    #[error("Missing code context for {classname}.{signature}")]
    MissingCodeContext {
        classname: String,
        signature: String,
    },

    #[error("Invalid array entry size of: {0}")]
    InvalidArrayEntrySize(usize),

    #[error("Attempted to access non-existing entry on array with index: {0}")]
    InvalidArrayAccess(usize),
}
