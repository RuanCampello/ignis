use crate::{
    classfile::{Classfile, ConstantPool, ConstantPoolEntry},
    vm::{
        Result, VmError,
        interpreter::StackFrame,
        runtime::{RuntimeError, heap::BaseInstance},
    },
};
use dashmap::DashMap;
use indexmap::IndexMap;
use once_cell::sync::{Lazy, OnceCell};
use parking_lot::RwLock;
use std::{collections::HashMap, fs::File, io::Read, ops::Index, path::Path, sync::Arc};

pub(in crate::vm) use class::{Class, FieldValue};

mod class;

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
pub(in crate::vm) struct Modules {
    registry: DashMap<String, i32>,
}

struct Pool<'c> {
    data: HashMap<PoolType, HashMap<u16, ConstantPoolEntry<'c>>>,
    pool: Vec<ConstantPoolEntry<'c>>,
    classname: Option<ClassName>,
}

struct ClassName {
    index: u16,
    name: String,
}

#[derive(Debug, Hash, PartialEq, Eq)]
enum PoolType {
    Empty,
    Utf8,
    Integer,
    Float,
    Long,
    Double,
    Class,
    String,
    Fieldref,
    Methodref,
    InterfaceMethodref,
    NameAndType,
    MethodHandle,
    MethodType,
    Dynamic,
    InvokeDynamic,
    Module,
    Package,
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

        let classname = match classname.starts_with('L') && classname.ends_with(';') {
            true => &classname[1..classname.len() - 1],
            _ => classname,
        };

        todo!("load from file")
    }

    pub fn create_instance_with_default(&self, classname: &str) -> Result<BaseInstance> {
        todo!()
    }

    pub(crate) fn fill_fields_hierarchy(
        &self,
        class_name: &str,
        instance_fields_hierarchy: &mut IndexMap<String, IndexMap<String, FieldValue>>,
    ) -> Result<()> {
        if instance_fields_hierarchy.contains_key(class_name) {
            return Ok(());
        }
        let rc = self.get(class_name)?;

        if let Some(parent_class_name) = rc.parent.as_ref() {
            self.fill_fields_hierarchy(parent_class_name, instance_fields_hierarchy)?;
        }

        let instance_fields = rc.default_value_fields();
        instance_fields_hierarchy.insert(class_name.to_string(), instance_fields.clone());

        Ok(())
    }

    fn load_from_file(&self, classname: &str) -> Result<Arc<Class>> {
        let filepath = format!("{classname}.class");

        // TODO: module parsing

        let mut file = match File::open(Path::new(&filepath)) {
            Ok(file) => Ok(file),
            Err(err) => Err(RuntimeError::FileLoadError {
                filepath,
                source: err,
            }),
        }?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer);

        unimplemented!()
    }

    fn try_parse(&self, buffer: &[u8]) -> Result<Option<Arc<Class>>> {
        let arena = bumpalo::Bump::new();
        let classfile = Classfile::new(buffer, &arena).expect("Failed to parse classfile");

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

        Arc::new(Class::with_classname(classname))
    }

    fn generate_class(classname: &str) -> Class {
        Class::with_classname(classname)
    }
}

impl Modules {
    fn new() -> Self {
        Self {
            registry: DashMap::new(),
        }
    }
}

impl<'c> Pool<'c> {
    fn new(pool: &[ConstantPoolEntry<'c>], classname: Option<ClassName>) -> Self {
        let mut data: HashMap<PoolType, HashMap<u16, ConstantPoolEntry<'c>>> = HashMap::new();

        for (idx, item) in pool.iter().enumerate() {
            let typ = item.into();
            let entry = data.entry(typ).or_insert_with(HashMap::new);
            entry.insert(idx as u16, item.clone());
        }

        Self {
            data,
            pool: pool.to_vec(),
            classname,
        }
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

impl<'c> From<&ConstantPoolEntry<'c>> for PoolType {
    fn from(value: &ConstantPoolEntry<'c>) -> Self {
        match value {
            ConstantPoolEntry::Utf8(_) => Self::Utf8,
            ConstantPoolEntry::Integer(_) => Self::Integer,
            ConstantPoolEntry::Float(_) => Self::Float,
            ConstantPoolEntry::Long(_) => Self::Long,
            ConstantPoolEntry::Double(_) => Self::Double,
            ConstantPoolEntry::Class(_) => Self::Class,
            ConstantPoolEntry::StringRef(_) => Self::String,
            ConstantPoolEntry::FieldRef(_, _) => Self::Fieldref,
            ConstantPoolEntry::MethodRef(_, _) => Self::Methodref,
            ConstantPoolEntry::InterfaceMethodRef(_, _) => Self::InterfaceMethodref,
            ConstantPoolEntry::NameAndType(_, _) => Self::NameAndType,
            ConstantPoolEntry::MethodHandle(_, _) => Self::MethodHandle,
            ConstantPoolEntry::MethodType(_) => Self::MethodType,
            ConstantPoolEntry::Dynamic(_, _) => Self::Dynamic,
            ConstantPoolEntry::InvokeDynamic(_, _) => Self::InvokeDynamic,
            ConstantPoolEntry::Module(_) => Self::Module,
            ConstantPoolEntry::Package(_) => Self::Package,
        }
    }
}
