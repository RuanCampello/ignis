//! This module implements the core of the Java Virtual Machine, providing the runtime environment necessary to execute Java bytecode.
//! It handles the execution of instructions defined by the JVM specification, managing the stack frames, operand stacks,
//! and local variables for each method invocation.
//!
//! The module is responsible for maintaining the lifecycle of objects, handling method calls and returns, and supporting control flow operations.
//! It also manages the interaction with the runtime constant pool and resolves symbolic references during execution.
//! This module acts as the bridge between the static class file data and the dynamic execution of Java programs,
//! forming the heart of the JVM interpreter and class loader runtime system.

#![allow(unused)]

use crate::vm::class::CLASSES;
use crate::vm::interpreter::executor::Executor;
use crate::vm::method_area::{class, with_method_area};
use crate::vm::runtime::RuntimeError;
use crate::vm::runtime::method_area::MethodArea;
use crate::{Args, vm::runtime::method_area};
use once_cell::sync::OnceCell;
use std::{
    path::{Path, PathBuf},
    sync::OnceLock,
};
use thiserror::Error;
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

mod descriptor;
mod interpreter;
mod launcher;
mod perfdata;
mod properties;
mod runtime;

#[derive(Error, Debug)]
pub enum VmError {
    #[error(transparent)]
    Runtime(#[from] runtime::RuntimeError),
    #[error(transparent)]
    Interpreter(#[from] interpreter::InterpreterError),
    #[error(transparent)]
    Image(#[from] crate::image::Error),
    #[error("I/O operation failed due to: {0}")]
    IO(#[from] std::io::Error),
    #[error("{0}")]
    Other(String),
    /// a java exception that propagated all the way up without being caught
    #[error("uncaught exception (throwable ref: {0})")]
    UncaughtException(i32),
}

static JAVA_HOME: OnceLock<PathBuf> = OnceLock::new();
static PLATAFORM_CLASS_LOADER: OnceLock<i32> = OnceLock::new();
static SYSTEM_CLASS_LAODER: OnceLock<i32> = OnceLock::new();
static CLASS_PATH: OnceCell<String> = OnceCell::new();

pub(in crate::vm) type Result<T> = std::result::Result<T, VmError>;

const UNSAFE_CONSTANTS: &str = "jdk/internal/misc/UnsafeConstants";
const THREAD_GROUP: &str = "java/lang/ThreadGroup";
const ACCESSIBLE_OBJ: &str = "java/lang/reflect/AccessibleObject";
const ADDRESS_SIZE: &str = "ADDRESS_SIZE0";

const SHUTDOWN: &str = "java/lang/Shutdown";
const SHUTDOWN_METHOD: &str = "shutdown:()V";

/// Launches the VM
/// This initialise the JVM itself, loading the given class and invoking it `main` function.
pub fn run(args: Args<'static>, java_home: impl AsRef<Path>) -> Result<()> {
    JAVA_HOME
        .set(java_home.as_ref().to_path_buf())
        .expect("JAVA_HOME was already set");

    let entry = args.entry;

    if entry.is_empty() {
        return Err(RuntimeError::Execution("entry class name cannot be empty".into()).into());
    }

    args.resolve_class_path()?;
    args.initialise_properties()?;
    args.initialise_perf_file()?;

    let result = (|| -> Result<()> {
        setup()?;

        let mode = match args.jar_mode {
            true => launcher::Mode::Jar,
            _ => launcher::Mode::Class,
        };

        launcher::execute_main(entry, mode, args.program_args.as_slice())?;
        Executor::static_method(SHUTDOWN, SHUTDOWN_METHOD, &[])?;

        Ok(())
    })();

    match result {
        Ok(()) => Ok(()),
        Err(err) => {
            if let Some(throwable_ref) = err.throwable_ref() {
                if let Err(err) = exception_handler(throwable_ref) {
                    tracing::error!("failed to invoke uncaught exception handler: {err}");
                }

                let shutdown = Executor::static_method("java/lang/Shutdown", "shutdown:()V", &[]);
                if let Err(err) = shutdown {
                    tracing::error!("failed to invoke shutdown hooks: {err}");
                }
            }

            Err(err)
        }
    }
}

fn exception_handler(throwable_ref: i32) -> Result<()> {
    let thread_id = with_method_area(|area| {
        area.thread_id
            .get()
            .copied()
            .expect("thread_id should be set by this point")
    });

    let thread_group_id = with_method_area(|area| {
        area.group_thread_id
            .get()
            .copied()
            .expect("group_thread_id should be set by this point")
    });

    let uncaught_exception = "uncaughtException:(Ljava/lang/Thread;Ljava/lang/Throwable;)V";
    Executor::non_static_method(
        "java/lang/ThreadGroup",
        uncaught_exception,
        thread_group_id,
        &[thread_id.into(), throwable_ref.into()],
    )?;

    Ok(())
}

fn setup() -> Result<()> {
    logger()?;
    MethodArea::initialise()?;

    class::CLASSES.pre()?;
    class::CLASSES.post()?;

    for class in MethodArea::generate_synthetic_classes() {
        CLASSES.insert(class, None)?;
    }

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

impl VmError {
    fn throwable_ref(&self) -> Option<i32> {
        match self {
            Self::UncaughtException(throwable) => Some(*throwable),
            _ => None,
        }
    }
}
