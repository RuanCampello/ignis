use crate::{
    classfile::{Classfile, ConstantPool, ConstantPoolEntry},
    image::image::Image,
    vm::{
        JAVA_HOME, Result, VmError,
        interpreter::{StackFrame, ldc::Ldc},
        runtime::{RuntimeError, heap::BaseInstance, method_area::class::CLASSES},
    },
};
use dashmap::DashMap;
use indexmap::IndexMap;
use once_cell::sync::{Lazy, OnceCell};
use parking_lot::{Mutex, RwLock};
use std::{
    collections::{HashMap, HashSet},
    fs::File,
    io::Read,
    ops::Index,
    path::Path,
    sync::Arc,
};

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
    image: Image,
    modules: Arc<Modules>,
    modules_map: HashMap<String, String>,
    ldc: Ldc,
    thread_id: OnceCell<i32>,
    /// Thread group created by the VM.
    group_thread_id: OnceCell<i32>,
}

#[derive(Debug)]
pub(in crate::vm) struct Modules {
    registry: DashMap<String, i32>,
    class_to_patch: Mutex<Option<HashSet<i32>>>,
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
            .set(MethodArea::new()?)
            .map_err(|_| RuntimeError::MethodAreaInitialised.into())
    }

    pub fn new<'a>() -> Result<Self> {
        let home = JAVA_HOME.get().ok_or_else(|| {
            RuntimeError::Execution("JAVA_HOME is not set, cannot initialise MethodArea".into())
        })?;

        let modules = home.join("lib").join("modules");
        let image = Image::open(modules)?;

        let modules_map = image
            .into_iter()
            .map(|result| result.map_err(From::from))
            .map(|result| result.map(|r| r.get_full_name()))
            .map(|result| result.map(|(module, name)| (name, module)))
            .collect::<Result<HashMap<_, _>>>()?;

        let modules = Arc::new(Modules::new());
        let ldc = Ldc::default();

        Ok(Self {
            image,
            modules,
            modules_map,
            ldc,
            thread_id: OnceCell::new(),
            group_thread_id: OnceCell::new(),
        })
    }

    pub fn get(&self, classname: &str) -> Result<Arc<Class>> {
        todo!()
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
}

impl Modules {
    fn new() -> Self {
        Self {
            registry: DashMap::new(),
            class_to_patch: Mutex::new(Some(HashSet::new())),
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

pub(in crate::vm::runtime::method_area) fn fill_fields_hierarchy(
    classname: &str,
    instance_fields_hierarchy: &mut IndexMap<String, IndexMap<String, FieldValue>>,
) -> Result<()> {
    let class = CLASSES.get(classname)?;
    if let Some(parent) = class.parent.as_ref() {
        fill_fields_hierarchy(parent, instance_fields_hierarchy)?;
    }

    let instance_fields = class.default_value_fields();
    instance_fields_hierarchy.insert(classname.to_string(), instance_fields);

    Ok(())
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
