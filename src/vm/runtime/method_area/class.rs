use crate::vm::{
    Result, VmError,
    interpreter::StackFrame,
    runtime::{
        RuntimeError,
        heap::{BaseInstance, ClassInstance, HEAP, Instance},
        method_area::{PRIMITIVE_TYPE, fill_fields_hierarchy, with_method_area},
    },
};
use dashmap::DashMap;
use indexmap::IndexMap;
use once_cell::sync::OnceCell;
use parking_lot::RwLock;
use std::{
    ops::DerefMut,
    sync::{
        Arc, LazyLock,
        atomic::{AtomicUsize, Ordering},
    },
};

#[derive(Debug, Default)]
pub(in crate::vm::runtime) struct Classes {
    classes: DashMap<String, Arc<ClassEntry>>,
    index: DashMap<usize, Arc<ClassEntry>>,
    next_id: AtomicUsize,
}

#[derive(Debug)]
pub(in crate::vm) struct Class {
    name: String,
    methods: IndexMap<String, Arc<Method>>,
    static_fields: IndexMap<String, Arc<FieldValue>>,
    pub(super) parent: Option<String>,
    modifiers: Modifier,

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

bitflags::bitflags! {
#[derive(Debug, Clone, Copy)]
struct Modifier: u16 {
    const Public     = 0x0001;
    const Private    = 0x0002;
    const Protected  = 0x0004;
    const Static     = 0x0008;
    const Final      = 0x0010;
    const Interface  = 0x0200;
    const Abstract   = 0x0400;
    const Strict     = 0x0800;
    const Synthetic  = 0x1000;
    const Annotation = 0x2000;
    const Enum       = 0x4000;
}}

pub(in crate::vm::runtime) static CLASSES: LazyLock<Classes> = LazyLock::new(Classes::default);

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
            true => Self::generate_synthetic_array(name),
            _ => with_method_area(|area| area.load_from_file(name))?,
        };

        self.insert(class, None)
    }

    pub fn get_by_id(&self, id: usize) -> Result<Arc<Class>> {
        self.index
            .get(&id)
            .map(|entry| Arc::clone(&entry.class))
            .ok_or_else(|| {
                RuntimeError::Execution(format!("class with id {id} was not found")).into()
            })
    }

    pub fn insert(&self, class: Arc<Class>, class_ref: Option<i32>) -> Result<ClassWithId> {
        let name = class.name.as_str();
        if let Some((id, name, class)) = self.get_impl(name) {
            return Ok((id, name.to_string(), class));
        }

        match !name.starts_with('[') {
            true => self.insert_class(&class, None, class_ref),
            _ => todo!(),
        }
    }

    fn insert_class(
        &self,
        class: &Arc<Class>,
        component_ref_type: Option<i32>,
        class_loader_ref: Option<i32>,
    ) -> Result<ClassWithId> {
        let name = class.name.as_str();
        if let Some((id, key, class)) = self.get_impl(name) {
            return Ok((id, key.to_string(), Arc::clone(&class)));
        }

        let class_id = self.insert_impl(name, Arc::clone(class));

        let (class_class_id, _, class_class) = self.get_impl(Class::NAME).ok_or_else(|| {
            RuntimeError::Execution(format!(
                "{} class was not found in the loaded classes",
                Class::NAME
            ))
        })?;

        Self::create_class_instance(
            (class, class_id),
            (&class_class, class_class_id),
            component_ref_type,
            class_loader_ref,
        )?;

        Ok((class_id, name.to_string(), Arc::clone(class)))
    }

    fn create_class_instance(
        class: (&Arc<Class>, usize),
        class_class: (&Arc<Class>, usize),
        component_ref_type: Option<i32>,
        class_loader_ref: Option<i32>,
    ) -> Result<()> {
        let (class, id) = class;
        let mut instance = Instance::Class(ClassInstance {
            class_id: id,
            instance: BaseInstance {
                id,
                fields: class.get_instance_fields()?.clone(),
            },
        });
        instance.set_field_value(
            Class::NAME,
            "componentType",
            vec![component_ref_type.unwrap_or(0)],
        )?;

        let primitive = PRIMITIVE_TYPE
            .contains_key(class.name.as_str())
            .then_some(1)
            .unwrap_or(0);
        instance.set_field_value(Class::NAME, "primitive", vec![primitive])?;

        let modifiers = class.modifiers.bits();
        instance.set_field_value(Class::NAME, "modifiers", vec![modifiers as i32])?;

        instance.set_field_value(
            Class::NAME,
            "classLoader",
            vec![class_loader_ref.unwrap_or(0)],
        )?;

        let (module, patch) = with_method_area(|area| {
            let file = format!("{}.class", class.name);
            match area.modules_map.get(&file) {
                Some(package) => {
                    let modules = &area.modules;
                    let registry = &modules.registry;
                    let module = registry.get(package).map(|v| *v.value()).unwrap_or(0);
                    let patch = package == "java.base" && module == 0;

                    (module, patch)
                }
                _ => {
                    todo!("unnamed module");
                }
            }
        });

        let class_instance_id = HEAP.allocate_instance(instance);

        with_method_area(|area| {
            if patch {
                let modules = &area.modules;
                let class_to_patch = &modules.class_to_patch;
                let mut guard = class_to_patch.lock();

                match guard.deref_mut() {
                    Some(to_patch) => to_patch.insert(class_instance_id),
                    _ => {
                        let err = RuntimeError::Execution("pathing was already executed".into());
                        return Err(err.into());
                    }
                };
            }

            Ok::<_, VmError>(())
        })?;

        // TODO: inject mirror class

        Ok(())
    }

    fn get_impl(&self, name: &str) -> Option<ClassWithId> {
        self.classes
            .get(name)
            .map(|entry| (entry.id, entry.key().to_string(), Arc::clone(&entry.class)))
    }

    fn insert_impl(&self, name: &str, class: Arc<Class>) -> usize {
        let entry = self.classes.entry(name.to_string()).or_insert_with(|| {
            let id = self.next_id.fetch_add(1, Ordering::SeqCst);
            let entry = Arc::new(ClassEntry { class, id });

            self.index.insert(id, Arc::clone(&entry));
            entry
        });

        entry.value().id
    }

    fn generate_synthetic_array(array_name: &str) -> Arc<Class> {
        let array_name = array_name.replace('/', ".");

        Arc::new(Class {
            name: array_name,
            parent: Some(Class::OBJECT.into()),
            modifiers: Modifier::Public | Modifier::Final | Modifier::Abstract,
            methods: IndexMap::new(),
            static_fields: IndexMap::new(),
            fields_schema: IndexMap::new(),
            fields_hierarchy: OnceCell::new(),
        })
    }
}

impl Class {
    const NAME: &str = "java/lang/Class";
    const OBJECT: &str = "java/lang/Object";

    pub fn with_classname(classname: &str) -> Self {
        Self {
            name: classname.to_string(),
            methods: IndexMap::new(),
            static_fields: IndexMap::new(),
            fields_schema: IndexMap::new(),
            fields_hierarchy: OnceCell::new(),
            modifiers: Modifier::empty(),
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

            with_method_area(|area| fill_fields_hierarchy(&self.name, &mut fields))?;
            Ok(fields)
        })
    }

    pub(super) fn default_value_fields(&self) -> IndexMap<String, FieldValue> {
        self.fields_schema.clone()
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
