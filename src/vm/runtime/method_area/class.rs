use crate::vm::{
    Result, VmError,
    interpreter::StackFrame,
    runtime::{RuntimeError, method_area::with_method_area},
};
use dashmap::DashMap;
use indexmap::IndexMap;
use once_cell::sync::OnceCell;
use parking_lot::RwLock;
use std::sync::Arc;

#[derive(Debug)]
pub(in crate::vm::runtime) struct Classes {
    classes: DashMap<String, ClassEntry>,
    index: DashMap<usize, Arc<ClassEntry>>,
}

#[derive(Debug)]
pub(in crate::vm) struct Class {
    name: String,
    methods: IndexMap<String, Arc<Method>>,
    static_fields: IndexMap<String, Arc<FieldValue>>,
    pub(super) parent: Option<String>,

    fields_hierarchy: OnceCell<IndexMap<String, IndexMap<String, FieldValue>>>,
    fields_schema: IndexMap<String, FieldValue>,
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
    pub value: RwLock<Vec<i32>>,
}

#[derive(Debug)]
struct ClassEntry {
    id: usize,
    class: Arc<Class>,
}

/// a class with its id and name
type ClassWithId = (usize, String, Arc<Class>);

impl Classes {
    pub fn is_loaded(&self, name: &str) -> bool {
        let name = undecorate_name(name);
        self.classes.contains_key(name)
    }

    /// tries to get a class by its name and loads it if necessary
    pub fn get(&self, name: &str) -> Result<Arc<Class>> {
        self.get_with_id(name).map(|(_, _, class)| class)
    }

    pub fn get_with_id(&self, name: &str) -> Result<ClassWithId> {
        let name = undecorate_name(name);

        if let Some((id, key, class)) = self.get_impl(name) {
            return Ok((id, key, class));
        }

        let class = match name.starts_with('[') {
            true => todo!(),
            _ => with_method_area(|area| area.load_from_file(name))?,
        };

        todo!()
    }

    pub fn get_by_id(&self, id: usize) -> Result<Arc<Class>> {
        self.index
            .get(&id)
            .map(|entry| Arc::clone(&entry.class))
            .ok_or_else(|| {
                RuntimeError::Execution(format!("class with id {id} was not found")).into()
            })
    }

    pub fn insert(&self, class: Arc<Class>, class_ref: Option<i16>) -> Result<ClassWithId> {
        let name = class.name.as_str();
        if let Some((id, name, class)) = self.get_impl(name) {
            return Ok((id, name.to_string(), class));
        }

        todo!()
    }

    fn create_class_instance(
        class: (&Arc<Class>, usize),
        class_class: (&Arc<usize>, usize),
        component_ref_type: Option<i32>,
        class_ref: Option<i32>,
    ) {
        todo!()
    }

    fn get_impl(&self, name: &str) -> Option<ClassWithId> {
        self.classes
            .get(name)
            .map(|entry| (entry.id, entry.key().to_string(), Arc::clone(&entry.class)))
    }
}

impl Class {
    pub fn with_classname(classname: &str) -> Self {
        Self {
            name: classname.to_string(),
            methods: IndexMap::new(),
            static_fields: IndexMap::new(),
            fields_schema: IndexMap::new(),
            fields_hierarchy: OnceCell::new(),
            parent: None,
        }
    }

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

    pub(super) fn get_instance_fields(
        &self,
    ) -> Result<&IndexMap<String, IndexMap<String, FieldValue>>> {
        self.fields_hierarchy.get_or_try_init(|| {
            let mut fields = IndexMap::new();

            with_method_area(|area| area.fill_fields_hierarchy(&self.name, &mut fields))?;
            Ok(fields)
        })
    }

    pub(super) fn default_value_fields(&self) -> &IndexMap<String, FieldValue> {
        &self.fields_schema
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
    pub(in crate::vm) fn value(&self) -> Result<Vec<i32>> {
        let guard = self.value.read();
        Ok(guard.clone())
    }

    pub fn set(&self, value: Vec<i32>) -> Result<()> {
        let mut guard = self.value.write();
        *guard = value;
        Ok(())
    }
}

impl Clone for FieldValue {
    fn clone(&self) -> Self {
        let value = self.value.read().clone();
        Self {
            value: RwLock::new(value),
        }
    }
}

#[inline(always)]
fn undecorate_name(name: &str) -> &str {
    match name.starts_with('L') && name.ends_with(';') {
        true => &name[1..name.len() - 1],
        _ => name,
    }
}
