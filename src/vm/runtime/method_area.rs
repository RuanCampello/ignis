use std::sync::Arc;

use dashmap::DashMap;
use indexmap::IndexMap;
use once_cell::sync::OnceCell;

static METHOD_AREA: OnceCell<MethodArea> = OnceCell::new();

#[derive(Debug)]
pub(in crate::vm::runtime) struct MethodArea<'c> {
    classes: DashMap<String, Class<'c>>,
}

#[derive(Debug)]
pub(in crate::vm::runtime) struct Class<'c> {
    methods: IndexMap<String, Arc<Method<'c>>>,
}

#[derive(Debug)]
pub(in crate::vm::runtime) struct Method<'m> {
    classname: Arc<&'m str>,
    signature: Arc<&'m str>,
    /// Indicates wheter a method is native or not.
    native: bool,

    annotations: Option<&'m [u8]>,
}

pub(crate) fn with_method_area<C, R>(callback: C) -> R
where
    C: FnOnce(&MethodArea) -> R,
{
    let area = METHOD_AREA.get().expect("Failed to get MethodArea");

    callback(&area)
}
