#![warn(unused_imports)]

use crate::vm::{
    Result, VmError,
    class::CLASSES,
    interpreter::{executor::Executor, static_method::Static},
    method_area::class,
    runtime::string_pool,
};

/// Derived from [openjdk](https://github.com/AdoptOpenJDK/openjdk-jdk11/blob/19fb8f93c59dfd791f62d41f332db9e306bc1422/src/java.base/share/classes/sun/launcher/LauncherHelper.java#L605-L612)
#[derive(Debug)]
#[repr(i32)]
pub(in crate::vm) enum Mode {
    Class = 1,
    Jar,
}

const PRINT_TO_STDERR: bool = true;

pub(in crate::vm) fn execute_main(classname: &str, mode: Mode, args: &[String]) -> Result<()> {
    let classname = string_pool::get(classname)?;

    let check_and_load = "checkAndLoadMain:(ZILjava/lang/String;)Ljava/lang/Class;";
    let helper = "sun/launcher/LauncherHelper";
    let check_and_load_args = [
        (PRINT_TO_STDERR as i32).into(),
        (mode as i32).into(),
        classname.into(),
    ];
    let class_ref = Executor::static_method(helper, check_and_load, &check_and_load_args)?[0];

    let class = class::get_class(class_ref)?;

    Static::initialise_class(&class)?;

    let helper = CLASSES.get(helper)?;
    let main_is_static = helper
        .get_static("isStaticMain")
        .ok_or_else(|| {
            VmError::Other("Failed to get isStaticMain field from LauncherHelper".into())
        })?
        .value()?[0]
        != 0;
    let main_has_no_arg = helper
        .get_static("noArgMain")
        .ok_or_else(|| VmError::Other("Failed to get noArgMain field from LauncherHelper".into()))?
        .value()?[0]
        != 0;

    let method_sig = match main_has_no_arg {
        true => "main:()V",
        _ => "main:([Ljava/lang/String;)V",
    };

    let args_array_ref = match !main_has_no_arg {
        true => Some(string_pool::create_string_array(args)?.into()),
        _ => None,
    };

    let args = match &args_array_ref {
        Some(arr) => std::slice::from_ref(arr),
        None => &[],
    };

    match main_is_static {
        true => Executor::static_method(&class.name, method_sig, args)?,
        _ => {
            let main_instance = Executor::default_constructor(&class.name)?;
            Executor::non_static_method(&class.name, method_sig, main_instance, args)?
        }
    };

    Ok(())
}
