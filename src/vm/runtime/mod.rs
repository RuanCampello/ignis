//! This module defines the JVM runtime data areas, including the heap, method area, and related
//! resources. It holds the live objects, class metadata, and constant pools needed during execution,
//! providing the dynamic state that the VM operates on.

use thiserror::Error;

mod heap;
mod method_area;

#[derive(Error, Debug)]
pub(self) enum VmError {
    #[error("METHOD_AREA was already initialised")]
    MethodAreaInitialised,
}
