use crate::vm::{
    Result,
    interpreter::{ValueRef, stack::Value},
    runtime::method_area::with_method_area,
};

// for as it now, executor is not going to hold any state
// but this may change in the future, for now it's going to be a
// more namespace delimiter
struct Executor {}

impl Executor {
    const INITIALISE_METHOD: &str = "<init>:()V";

    fn execute<'a>(classname: &str, method_name: &str, args: &[Value]) -> Result<Vec<ValueRef>> {
        let class = with_method_area(|area| area.get(classname))?;
        let method = class.get_method(method_name)?;
        let mut frame = method.new_frame()?;
        // TODO: set args

        super::execute(frame)
    }
}
