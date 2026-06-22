use std::sync::Arc;

use crate::vm::{
    Result,
    interpreter::{self, StackFrame, ValueRef, stack::Value},
    method_area::{CLASSES, Class},
    runtime::{self, heap::HEAP, method_area::with_method_area},
};

// for as it now, executor is not going to hold any state
// but this may change in the future, for now it's going to be a
// more namespace delimiter
pub(in crate::vm) struct Executor {}

impl Executor {
    const INITIALISE_METHOD: &str = "<init>:()V";

    fn execute<'a>(classname: &str, method_name: &str, args: &[Value]) -> Result<Vec<ValueRef>> {
        let class = with_method_area(|area| area.get(classname))?;
        let method = class.get_method(method_name)?;
        let mut frame = method.new_frame()?;
        // TODO: set args

        super::execute(frame)
    }

    pub fn default_constructor(classname: &str) -> Result<ValueRef> {
        todo!()
    }

    pub(in crate::vm) fn static_method(
        classname: &str,
        method_name: &str,
        args: &[Value],
    ) -> Result<Vec<ValueRef>> {
        let class = CLASSES.get(classname)?;

        Self::execute_for_class(&class, method_name, args, None)
    }

    pub(in crate::vm) fn constructor(
        classname: &str,
        method_name: &str,
        args: &[Value],
    ) -> Result<ValueRef> {
        let class = CLASSES.get(classname)?;
        let instance = CLASSES.new_base_instance(classname)?;
        let reference = HEAP.allocate_instance(instance);

        let mut arguments = Vec::with_capacity(args.len() + 1);
        arguments.push(Value::from(reference));
        arguments.extend_from_slice(args);

        Self::execute_for_class(&class, method_name, &arguments, None)?;

        Ok(reference)
    }

    fn execute_for_class(
        class: &Arc<Class>,
        method_name: &str,
        args: &[Value],
        reason: Option<&str>,
    ) -> Result<Vec<ValueRef>> {
        let method = class.get_method(method_name)?;
        let mut stack_frame = method.new_frame()?;

        Self::set_stack(&mut stack_frame, args)?;

        interpreter::execute(stack_frame)
    }

    fn set_stack(stack_frame: &mut StackFrame, args: &[Value]) -> Result<()> {
        let mut chunk_index = 0;

        for arg in args.iter() {
            match arg {
                Value::Int(value) => stack_frame.set(chunk_index, *value),
                Value::Long(value) => stack_frame.set(chunk_index, *value),
                Value::Float(value) => stack_frame.set(chunk_index, *value),
                Value::Double(value) => stack_frame.set(chunk_index, *value),
            }

            chunk_index += arg.chunks();
        }

        Ok(())
    }
}
