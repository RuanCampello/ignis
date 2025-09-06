use crate::vm::{
    Result,
    runtime::method_area::{Class, with_method_area},
};

pub(in crate::vm) struct Static {}

impl Static {
    const STATIC_INIT_METHOD: &'static str = "<clinit>:()V";

    pub fn initialise(classname: &str) -> Result<()> {
        let class = with_method_area(|area| area.get(classname))?;
        todo!()
    }

    fn initialise_class(class: &Class) -> Result<()> {
        todo!()
    }
}
