use dashmap::DashMap;
use indexmap::IndexMap;
use once_cell::sync::{Lazy, OnceCell};
use parking_lot::RwLock;
use std::{collections::HashMap, path::Path, sync::Arc};

use crate::vm::runtime::VmError;

static METHOD_AREA: OnceCell<MethodArea> = OnceCell::new();
static PRIMITIVE_TYPE: Lazy<HashMap<&str, &str>> = {
    Lazy::new(|| {
        let mut hm = HashMap::new();
        hm.insert("B", "byte");
        hm.insert("C", "char");
        hm.insert("D", "double");
        hm.insert("F", "float");
        hm.insert("I", "int");
        hm.insert("J", "long");
        hm.insert("S", "short");
        hm.insert("Z", "boolean");
        hm.insert("V", "void");
        hm
    })
};

#[derive(Debug)]
pub(in crate::vm::runtime) struct MethodArea<'c> {
    classes: DashMap<String, Class<'c>>,
    reflection: DashMap<i32, String>,
    thread_id: OnceCell<i32>,
    /// Thread group created by the VM.
    group_thread_id: OnceCell<i32>,
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

#[derive(Debug)]
pub(in crate::vm::runtime) struct FieldValue {
    value: RwLock<Vec<i32>>,
}

pub(crate) fn with_method_area<C, R>(callback: C) -> R
where
    C: FnOnce(&MethodArea) -> R,
{
    let area = METHOD_AREA.get().expect("Failed to get MethodArea");

    callback(&area)
}

impl<'m> MethodArea<'m> {
    pub(super) fn initialise(path: impl AsRef<Path>) -> Result<(), VmError> {
        METHOD_AREA
            .set(MethodArea::new(path)?)
            .map_err(|_| VmError::MethodAreaInitialised)
    }

    pub(super) fn new(path: impl AsRef<Path>) -> Result<Self, VmError> {
        let modules = path.as_ref().join("lib").join("modules");

        Ok(Self {
            classes: DashMap::new(),
            reflection: DashMap::new(),
            thread_id: OnceCell::new(),
            group_thread_id: OnceCell::new(),
        })
    }
}
