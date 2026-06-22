#![warn(unused_imports)]

use crate::classfile::{Classfile, FieldFlags, MethodFlags};
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
        atomic::{AtomicI8, AtomicUsize, Ordering},
    },
};
use tracing::trace;

#[derive(Debug, Default)]
pub(in crate::vm) struct Classes {
    classes: DashMap<String, Arc<ClassEntry>>,
    index: DashMap<usize, Arc<ClassEntry>>,
    next_id: AtomicUsize,

    stage: AtomicI8,
}

#[derive(Debug)]
pub(in crate::vm) struct Class {
    name: String,
    methods: IndexMap<String, Arc<Method>>,
    static_fields: IndexMap<String, Arc<FieldValue>>,
    pub(super) parent: Option<String>,
    pub(super) external_name: String,
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

pub(in crate::vm) static CLASSES: LazyLock<Classes> = LazyLock::new(Classes::default);

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

    #[inline]
    pub(in crate::vm) fn new_instance(&self, name: &str) -> Result<Instance> {
        let (id, _, class) = self.get_with_id(name)?;

        Ok(Instance::Base(BaseInstance {
            id,
            fields: class.get_instance_fields()?.clone(),
        }))
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

    /// pre-construction initialisation, used to load `Object` and `Class` before any
    /// other classes are loaded
    pub fn pre(&self) -> Result<()> {
        let stage = self.stage.fetch_add(1, Ordering::SeqCst);

        match stage == 0 {
            true => {
                let object = with_method_area(|area| area.load_from_file(Class::OBJECT))?;
                self.insert_impl(Class::OBJECT, Arc::clone(&object));

                trace!("CLASS LOADED -> {}", Class::OBJECT);

                let class = with_method_area(|area| area.load_from_file(Class::NAME))?;
                self.insert_impl(Class::NAME, Arc::clone(&class));

                trace!("CLASS LOADED -> {}", Class::NAME);

                Ok(())
            }

            _ => Err(RuntimeError::Execution(format!(
                "pre-construction was invoked at the wrong construction stage: {stage}"
            ))
            .into()),
        }
    }

    /// post-construction initialisation, used to create `Clas` and `Object` to avoid
    /// infinity recursion during class loading
    pub fn post(&self) -> Result<()> {
        let stage = self.stage.fetch_add(1, Ordering::SeqCst);

        match stage == 1 {
            true => {
                let (class_id, name, class) = self
                    .get_impl(Class::NAME)
                    .expect("CLASS must be loaded in pre-construction phase");

                Self::create_class_instance((&class, class_id), (&class, class_id), None, None)?;

                let (object_id, name, object) = self
                    .get_impl(Class::OBJECT)
                    .expect("OBJECT must be loaded in pre-construction phase");

                Self::create_class_instance((&object, object_id), (&class, class_id), None, None)?;

                Ok(())
            }
            _ => Err(RuntimeError::Execution(format!(
                "post-construction was invoked at the wrong construction stage: {stage}"
            ))
            .into()),
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

    fn generate_synthetic_array(internal: &str) -> Arc<Class> {
        let external = internal.replace('/', ".");

        Arc::new(Class {
            name: internal.to_string(),
            external_name: external,
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
    pub(in crate::vm::runtime) const NAME: &str = "java/lang/Class";
    const OBJECT: &str = "java/lang/Object";

    pub fn with_classname(classname: &str) -> Self {
        Self {
            name: classname.to_string(),
            methods: IndexMap::new(),
            static_fields: IndexMap::new(),
            fields_schema: IndexMap::new(),
            fields_hierarchy: OnceCell::new(),
            modifiers: Modifier::empty(),
            external_name: String::default(),
            parent: None,
        }
    }

    /// Builds a runtime [Class] out of a freshly parsed [Classfile].
    ///
    /// `name` is the already-resolved internal name of the class, see
    /// [internal_and_external_names](super::internal_and_external_names).
    pub(super) fn from_classfile(
        classfile: &Classfile,
        name: &str,
        external: String,
    ) -> Result<Self> {
        let pool = classfile.constant_pool;

        let parent = classfile.super_class().map(ToString::to_string);
        let modifiers = Modifier::from_bits_truncate(classfile.access_flags());
        let classname: Arc<str> = Arc::from(name);

        let mut methods = IndexMap::with_capacity(classfile.methods.len());
        for method in classfile.methods {
            let name = method
                .name(pool)
                .map_err(|e| RuntimeError::Execution(e.to_string()))?;
            let descriptor = method
                .descriptor(pool)
                .map_err(|e| RuntimeError::Execution(e.to_string()))?;
            let flags = method.flags();

            let signature: Arc<str> = Arc::from(format!("{name}:{descriptor}").as_str());
            let native = flags.contains(MethodFlags::NATIVE);

            // abstract and native methods carry no bytecode of their own.
            let context = match flags.intersects(MethodFlags::ABSTRACT | MethodFlags::NATIVE) {
                true => None,
                false => method.code().map(|(max_stack, max_locals, code)| Context {
                    max_stack,
                    max_locals,
                    bytecode: Arc::from(code),
                }),
            };
            let annotations = method.annotations().map(<[u8]>::to_vec);

            methods.insert(
                signature.to_string(),
                Arc::new(Method {
                    classname: Arc::clone(&classname),
                    signature,
                    context,
                    native,
                    annotations,
                }),
            );
        }

        let mut static_fields = IndexMap::new();
        let mut fields_schema = IndexMap::new();
        for field in classfile.fields {
            let name = field
                .name(pool)
                .map_err(|e| RuntimeError::Execution(e.to_string()))?;
            let descriptor = field
                .descriptor(pool)
                .map_err(|e| RuntimeError::Execution(e.to_string()))?;

            // TODO: honour `ConstantValue` for static finals instead of defaulting.
            let value = FieldValue::default_for(descriptor);

            match field.flags().contains(FieldFlags::STATIC) {
                true => {
                    static_fields.insert(name.to_string(), Arc::new(value));
                }
                false => {
                    fields_schema.insert(name.to_string(), value);
                }
            }
        }

        Ok(Self {
            name: name.to_string(),
            external_name: external,
            methods,
            static_fields,
            parent,
            modifiers,
            fields_hierarchy: OnceCell::new(),
            fields_schema,
        })
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
    /// Zero-initialised value sized after a field `descriptor`: `long` and `double`
    /// take two slots, every other type a single one.
    fn default_for(descriptor: &str) -> Self {
        let slots = match descriptor.starts_with(['J', 'D']) {
            true => 2,
            false => 1,
        };

        Self {
            value: RwLock::new(vec![0; slots]),
        }
    }

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
