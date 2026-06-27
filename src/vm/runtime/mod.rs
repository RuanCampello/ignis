//! This module defines the JVM runtime data areas, including the heap, method area, and related
//! resources. It holds the live objects, class metadata, and constant pools needed during execution,
//! providing the dynamic state that the VM operates on.

use thiserror::Error;

pub(in crate::vm) mod heap;
pub(in crate::vm) mod method_area;
pub(in crate::vm) mod string_pool;

/// errors from the runtime data areas: class loading, the method area and the heap
///
/// this is the loading/linking-side umbrella under [`VmError`](crate::vm::VmError),
/// everything here is raised resolving classes, fields, methods or array/heap state,
/// never while decoding an opcode (that's [`InterpreterError`](crate::vm::interpreter::InterpreterError))
#[derive(Error, Debug)]
pub enum RuntimeError {
    /// the method area was set up twice
    #[error("METHOD_AREA was already initialised")]
    MethodAreaInitialised,

    /// reading a class out of the JDK's `lib/modules` jimage failed during loading
    #[error(transparent)]
    Image(#[from] crate::image::Error),

    /// method resolution found no method with this signature
    #[error("Method with signature {0} does not exists")]
    MethodNotFound(String),

    /// field access named a field the object's class does not declare
    #[error("Attempted to access non-existing field: '{field}' of object of class '{classname}'")]
    InvalidObjectAcess { classname: String, field: String },

    /// a method was invoked but carries no bytecode (abstract/native with no binding)
    #[error("Missing code context for {classname}.{signature}")]
    MissingCodeContext {
        classname: String,
        signature: String,
    },

    /// array element width could not be determined
    #[error("Invalid array entry size of: {0}")]
    InvalidArrayEntrySize(usize),

    /// indexed past the bounds of an array
    #[error("Attempted to access non-existing entry on array with index: {0}")]
    InvalidArrayAccess(usize),

    /// a classfile could not be read off disk
    #[error("Failed to open class file '{filepath}': {source}")]
    FileLoadError {
        filepath: String,
        source: std::io::Error,
    },

    /// catch-all for loading/linking failures without a dedicated variant
    #[error("Failed execution due to: {0}")]
    Execution(String),
}
