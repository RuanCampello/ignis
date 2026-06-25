//! This module implements the core of the Java Virtual Machine, providing the runtime environment necessary to execute Java bytecode.
//! It handles the execution of instructions defined by the JVM specification, managing the stack frames, operand stacks,
//! and local variables for each method invocation.
//!
//! The module is responsible for maintaining the lifecycle of objects, handling method calls and returns, and supporting control flow operations.
//! It also manages the interaction with the runtime constant pool and resolves symbolic references during execution.
//! This module acts as the bridge between the static class file data and the dynamic execution of Java programs,
//! forming the heart of the JVM interpreter and class loader runtime system.

#![allow(unused)]

use crate::vm::class::{CLASSES, Class};
use crate::vm::interpreter::{executor::Executor, static_method::Static};
use crate::vm::method_area::{class, with_method_area};
use crate::vm::runtime::method_area::MethodArea;
use crate::vm::runtime::{RuntimeError, string_pool};
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
static SYSTEM_CLASS_LOADER: OnceLock<i32> = OnceLock::new();
static CLASS_PATH: OnceCell<String> = OnceCell::new();
pub(in crate::vm) static UNNAMED_MODULE: OnceLock<i32> = OnceLock::new();

pub(in crate::vm) type Result<T> = std::result::Result<T, VmError>;

const UNSAFE_CONSTANTS: &str = "jdk/internal/misc/UnsafeConstants";
const THREAD_GROUP: &str = "java/lang/ThreadGroup";
const ACCESSIBLE_OBJ: &str = "java/lang/reflect/AccessibleObject";
const REFLECT_CLASS: &str = "java/lang/reflect/Method";
const SYSTEM_CLASS: &str = "java/lang/System";
const CLASS_LOADER: &str = "java/lang/ClassLoader";

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
    patch_class_mirror_fields()?;
    class::CLASSES.post()?;

    for class in MethodArea::generate_synthetic_classes() {
        CLASSES.insert(class, None)?;
    }

    initialise()?;

    Ok(())
}

fn initialise() -> Result<()> {
    const RESOLVED_METHOD_NAME: &str = "java/lang/invoke/ResolvedMethodName";
    const VM_TARGET: &str = "vmtarget";

    put_synthetic_field(
        RESOLVED_METHOD_NAME,
        VM_TARGET,
        descriptor::TypeDescriptor::Long,
    )?;

    Static::initialise(UNSAFE_CONSTANTS)?;

    let unsafe_constants = CLASSES.get(UNSAFE_CONSTANTS)?;
    let set_constant = |field: &str, value: i32| -> Result<()> {
        unsafe_constants
            .get_static(field)
            .ok_or_else(|| {
                RuntimeError::Execution(format!("{UNSAFE_CONSTANTS}.{field} is missing"))
            })?
            .set(vec![value])
    };

    set_constant("BIG_ENDIAN", cfg!(target_endian = "big") as i32)?;
    set_constant(ADDRESS_SIZE, std::mem::size_of::<usize>() as i32)?;
    set_constant("PAGE_SIZE", page_size::get() as i32)?;

    let thread_group = Executor::default_constructor(THREAD_GROUP)?;
    with_method_area(|area| {
        area.group_thread_id
            .set(thread_group)
            .expect("thread_group_id was already set")
    });

    let thread_name = string_pool::get("system")?;
    Executor::primordial_thread(&[thread_group.into(), thread_name.into()])?;

    Static::initialise(REFLECT_CLASS)?;
    Executor::static_method(SYSTEM_CLASS, "initPhase1:()V", &[])?;
    let init_phase =
        Executor::static_method(SYSTEM_CLASS, "initPhase2:(ZZ)I", &[1.into(), 1.into()])?[0];

    if init_phase != 0 {
        return Err(RuntimeError::Execution(format!(
            "System.initPhase2 returned an error: {init_phase}"
        ))
        .into());
    }

    Executor::static_method(SYSTEM_CLASS, "initPhase3:()V", &[])?;

    PLATAFORM_CLASS_LOADER
        .set(
            Executor::static_method(
                CLASS_LOADER,
                "getPlatformClassLoader:()Ljava/lang/ClassLoader;",
                &[],
            )?[0],
        )
        .expect("PLATAFORM_CLASS_LOADER must not be set");

    SYSTEM_CLASS_LOADER
        .set(
            Executor::static_method(
                CLASS_LOADER,
                "getSystemClassLoader:()Ljava/lang/ClassLoader;",
                &[],
            )?[0],
        )
        .expect("SYSTEM_CLASS_LOADER must be not set");

    let system_class_ref = SYSTEM_CLASS_LOADER
        .get()
        .copied()
        .expect("SYSTEM_CLASS_LOADER must be set");

    let module_ref = Executor::constructor(
        "java/lang/Module",
        "<init>:(Ljava/lang/ClassLoader;)V",
        &[system_class_ref.into()],
    )?;

    UNNAMED_MODULE
        .set(module_ref)
        .expect("UNNAMED_MODULE must not be set");

    Ok(())
}

fn patch_class_mirror_fields() -> Result<()> {
    use descriptor::TypeDescriptor;

    put_synthetic_field(Class::CLASS, "primitive", TypeDescriptor::Boolean)?;
    put_synthetic_field(Class::CLASS, "modifiers", TypeDescriptor::Integer)?;

    Ok(())
}

fn put_synthetic_field(
    classname: &str,
    name: &str,
    descriptor: descriptor::TypeDescriptor,
) -> Result<()> {
    let class = CLASSES.get(classname)?;
    let class = std::sync::Arc::into_raw(class) as *mut class::Class;

    let result = unsafe { (*class).put_instance_field(name.to_string(), descriptor, 0, classname) };

    // SAFETY: rebuild the `Arc` we turned into a raw pointer above so its strong count stays
    // balanced, `CLASSES` still holds its own reference, so this never frees the class
    let _ = unsafe { std::sync::Arc::from_raw(class) };

    if let Some(existing) = result? {
        return Err(RuntimeError::Execution(format!(
            "field {name}:{} already exists in {classname}",
            existing.descriptor()
        ))
        .into());
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
