//! This module implements the core of the Java Virtual Machine, providing the runtime environment necessary to execute Java bytecode.
//! It handles the execution of instructions defined by the JVM specification, managing the stack frames, operand stacks,
//! and local variables for each method invocation.
//!
//! The module is responsible for maintaining the lifecycle of objects, handling method calls and returns, and supporting control flow operations.
//! It also manages the interaction with the runtime constant pool and resolves symbolic references during execution.
//! This module acts as the bridge between the static class file data and the dynamic execution of Java programs,
//! forming the heart of the JVM interpreter and class loader runtime system.

use std::path::Path;
use thiserror::Error;
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

mod interpreter;
mod runtime;

#[derive(Default)]
struct Args {
    entry: String,
}

#[derive(Error, Debug)]
enum VmError {
    #[error(transparent)]
    Runtime(#[from] runtime::RuntimeError),
}

pub(in crate::vm) type Result<T> = std::result::Result<T, VmError>;

/// Launches the VM.
/// This initialise the JVM itself, loading the given class and invoking it `main` function.
fn run(path: &Path) -> Result<()> {
    todo!()
}

/// Initialise the logger.
fn logger() -> Result<()> {
    let layer = fmt::layer().with_target(false).with_ansi(false);
    let env_layer = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("info"))
        .expect("Couldn't create EnvFilter");

    tracing_subscriber::registry()
        .with(layer)
        .with(env_layer)
        .init();

    Ok(())
}
