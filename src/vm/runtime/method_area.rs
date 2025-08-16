use crate::vm::{Result, VmError, runtime::RuntimeError};
use dashmap::DashMap;
use indexmap::IndexMap;
use once_cell::sync::{Lazy, OnceCell};
use parking_lot::RwLock;
use std::{collections::HashMap, path::Path, sync::Arc};

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
    const PUBLIC: u16 = 0x0001;
    const ABSTRACT: u16 = 0x0400;
    const FINAL: u16 = 0x0010;

    pub fn initialise(path: impl AsRef<Path>) -> Result<()> {
        METHOD_AREA
            .set(MethodArea::new(path)?)
            .map_err(|_| RuntimeError::MethodAreaInitialised.into())
    }

    pub fn new<'a>(path: impl AsRef<Path>) -> Result<Self> {
        let modules = path.as_ref().join("lib").join("modules");
        let classes = Self::generate_classes();

        Ok(Self {
            classes,
            reflection: DashMap::new(),
            thread_id: OnceCell::new(),
            group_thread_id: OnceCell::new(),
        })
    }

    fn generate_classes<'c>() -> DashMap<String, Class<'c>> {
        PRIMITIVE_TYPE
            .keys()
            .map(|class_name| (class_name.to_string(), Self::generate_class(class_name)))
            .collect()
    }

    fn generate_class(classname: &str) -> Class {
        Class {
            methods: IndexMap::new(),
        }
    }
}

impl FieldValue {
    pub(super) fn value(&self) -> Result<Vec<i32>> {
        let guard = self.value.read();
        Ok(guard.clone())
    }
}
