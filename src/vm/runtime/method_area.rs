use crate::vm::{Result, VmError, interpreter::StackFrame, runtime::RuntimeError};
use dashmap::DashMap;
use indexmap::IndexMap;
use once_cell::sync::{Lazy, OnceCell};
use parking_lot::RwLock;
use std::{collections::HashMap, ops::Index, path::Path, sync::Arc};

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
pub(in crate::vm) struct MethodArea {
    classes: DashMap<String, Arc<Class>>,
    reflection: DashMap<i32, String>,
    thread_id: OnceCell<i32>,
    /// Thread group created by the VM.
    group_thread_id: OnceCell<i32>,
}

#[derive(Debug)]
pub(in crate::vm) struct Class {
    methods: IndexMap<String, Arc<Method>>,
    static_fields: IndexMap<String, Arc<FieldValue>>,
}

#[derive(Debug)]
pub(in crate::vm) struct Method {
    classname: Arc<str>,
    signature: Arc<str>,
    context: Option<Context>,
    /// Indicates wheter a method is native or not.
    native: bool,

    annotations: Option<Vec<u8>>,
}

#[derive(Debug)]
pub(in crate::vm) struct Context {
    max_stack: u16,
    max_locals: u16,
    bytecode: Arc<[u8]>,
}

#[derive(Debug)]
pub(in crate::vm) struct FieldValue {
    value: RwLock<Vec<i32>>,
}

pub(crate) fn with_method_area<C, R>(callback: C) -> R
where
    C: FnOnce(&MethodArea) -> R,
{
    let area = METHOD_AREA.get().expect("Failed to get MethodArea");

    callback(&area)
}

impl MethodArea {
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

    pub fn get(&self, classname: &str) -> Result<Arc<Class>> {
        if let Some(class) = self.classes.get(classname) {
            return Ok(Arc::clone(class.value()));
        }

        if classname.starts_with('[') {
            let class = Self::generate_array_class(classname);
            self.classes
                .insert(classname.to_string(), Arc::clone(&class));

            return Ok(class);
        }

        // TODO: load from file
        todo!()
    }

    fn generate_classes() -> DashMap<String, Arc<Class>> {
        PRIMITIVE_TYPE
            .keys()
            .map(|class_name| {
                (
                    class_name.to_string(),
                    Arc::new(Self::generate_class(class_name)),
                )
            })
            .collect()
    }

    fn generate_array_class(classname: &str) -> Arc<Class> {
        let (internal, external) = internal_and_external_names(classname);

        Arc::new(Class {
            methods: IndexMap::new(),
            static_fields: IndexMap::new(),
        })
    }

    fn generate_class(classname: &str) -> Class {
        Class {
            methods: IndexMap::new(),
            static_fields: IndexMap::new(),
        }
    }
}

impl Class {
    pub fn get_method(&self, signature: &str) -> Result<Arc<Method>> {
        self.get_full_method(signature)
            .and_then(|(_, method)| Some(method))
            .ok_or(RuntimeError::MethodNotFound(signature.into()).into())
    }

    fn get_full_method(&self, signature: &str) -> Option<(usize, Arc<Method>)> {
        self.methods
            .get_full(signature)
            .map(|(idx, _, method)| (idx, method.clone()))
            .or_else(|| {
                self.methods
                    .get_full(signature.split(":").next()?)
                    .map(|(idx, _, method)| (idx, method.clone()))
            })
    }

    pub fn get_static(&self, static_field: &str) -> Option<Arc<FieldValue>> {
        self.static_fields
            .get(static_field)
            .map(|field| Arc::clone(field))
    }
}

impl Method {
    pub fn new_frame(&self) -> Result<StackFrame> {
        match &self.context {
            Some(ctx) => Ok(StackFrame::new(
                ctx.max_locals as usize,
                ctx.max_stack as usize,
                Arc::clone(&ctx.bytecode),
                Arc::clone(&self.classname),
            )),
            None => Err(RuntimeError::MissingCodeContext {
                classname: self.classname.to_string(),
                signature: self.signature.to_string(),
            }
            .into()),
        }
    }
}

impl FieldValue {
    pub(super) fn value(&self) -> Result<Vec<i32>> {
        let guard = self.value.read();
        Ok(guard.clone())
    }

    pub fn set(&self, value: Vec<i32>) -> Result<()> {
        let mut guard = self.value.write();
        *guard = value;
        Ok(())
    }
}

fn internal_and_external_names(string: &str) -> (String, String) {
    const SYNTH_CLASS_DELIM: &str = "#";
    if let Some(external) = PRIMITIVE_TYPE.get(string) {
        return (string.to_string(), external.to_string());
    }

    match string.rsplit_once(SYNTH_CLASS_DELIM) {
        Some((base, suffix)) => {
            let internal = format!("{}/{}", base, suffix);
            let external = format!("{}/{}", base.replace('/', "."), suffix);
            (internal, external)
        }
        None => {
            let internal = string.to_string();
            let external = string.replace('/', ".");
            (internal, external)
        }
    }
}
