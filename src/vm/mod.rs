//! This module implements the core of the Java Virtual Machine, providing the runtime environment necessary to execute Java bytecode.
//! It handles the execution of instructions defined by the JVM specification, managing the stack frames, operand stacks,
//! and local variables for each method invocation.
//!
//! The module is responsible for maintaining the lifecycle of objects, handling method calls and returns, and supporting control flow operations.
//! It also manages the interaction with the runtime constant pool and resolves symbolic references during execution.
//! This module acts as the bridge between the static class file data and the dynamic execution of Java programs,
//! forming the heart of the JVM interpreter and class loader runtime system.

#![allow(unused)]

use crate::Args;
use crate::vm::{
    interpreter::{executor::Executor, static_method::Static},
    runtime::method_area::{MethodArea, with_method_area},
};
use std::{
    path::{Path, PathBuf},
    sync::OnceLock,
};
use thiserror::Error;
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

mod descriptor;
mod interpreter;
mod runtime;

#[derive(Error, Debug)]
pub enum VmError {
    #[error(transparent)]
    Runtime(#[from] runtime::RuntimeError),
    #[error(transparent)]
    Interpreter(#[from] interpreter::InterpreterError),
    #[error(transparent)]
    Image(#[from] crate::image::Error),
}

static JAVA_HOME: OnceLock<PathBuf> = OnceLock::new();
static PLATAFORM_CLASS_LOADER: OnceLock<i32> = OnceLock::new();
static SYSTEM_CLASS_LAODER: OnceLock<i32> = OnceLock::new();

pub(in crate::vm) type Result<T> = std::result::Result<T, VmError>;

const UNSAFE_CONSTANTS: &str = "jdk/internal/misc/UnsafeConstants";
const THREAD_GROUP: &str = "java/lang/ThreadGroup";
const ACCESSIBLE_OBJ: &str = "java/lang/reflect/AccessibleObject";
const ADDRESS_SIZE: &str = "ADDRESS_SIZE0";

/// Launches the VM
/// This initialise the JVM itself, loading the given class and invoking it `main` function.
pub fn run(args: Args, java_home: impl AsRef<Path>) -> Result<()> {
    JAVA_HOME
        .set(java_home.as_ref().to_path_buf())
        .expect("JAVA_HOME was already set");

    todo!()
}

fn setup(path: &Path) -> Result<()> {
    logger()?;
    MethodArea::initialise(path)?;

    Ok(())
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
