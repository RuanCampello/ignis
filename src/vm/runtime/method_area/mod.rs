use crate::{
    classfile::Classfile,
    image::image::Image,
    vm::{
        JAVA_HOME, Result, VmError,
        interpreter::{StackFrame, executor::Executor, ldc::Ldc},
        method_area::class::{CLASSES, Class, FieldValue},
        runtime::{
            RuntimeError,
            heap::{BaseInstance, HEAP},
        },
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
    path::{Path, PathBuf},
    sync::Arc,
};

pub(in crate::vm) mod class;

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
    pub(in crate::vm) thread_id: OnceCell<i32>,
    /// Thread group created by the VM.
    pub(in crate::vm) group_thread_id: OnceCell<i32>,
}

#[derive(Debug)]
pub(in crate::vm) struct Modules {
    registry: DashMap<String, i32>,
    class_to_patch: Mutex<Option<HashSet<i32>>>,
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

    pub fn initialise() -> Result<()> {
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

    pub(in crate::vm) fn generate_synthetic_classes() -> Vec<Arc<Class>> {
        PRIMITIVE_TYPE
            .keys()
            .map(|classname| Arc::new(Class::new_synthetic(classname)))
            .collect()
    }

    fn load_from_file(&self, classname: &str) -> Result<Arc<Class>> {
        let class_path = format!("{classname}.class");

        if let Some(module) = self.modules_map.get(&class_path) {
            let resource = format!("/{module}/{class_path}");

            if let Some(resource) = self
                .image
                .find_resource(&resource)
                .map_err(|image| RuntimeError::Execution(image.to_string()))?
            {
                match self.parse(&resource) {
                    Ok(Some(class)) => return Ok(class),
                    Ok(None) => {}
                    Err(e) => return Err(e),
                }
            }
        }

        match class_path.starts_with("java/") {
            true => self.open_and_parse(&class_path)?.ok_or_else(|| {
                RuntimeError::Execution(format!("error opening class file: {class_path}")).into()
            }),
            _ => {
                let external = classname.replace('/', ".");

                let for_name =
                    "forName:(Ljava/lang/String;ZLjava/lang/ClassLoader;)Ljava/lang/Class;";
                let class_ref = Executor::static_method(Class::CLASS, for_name, &[])?;

                todo!("load application class `{external}` via Class.forName")
            }
        }
    }

    fn parse(&self, buff: &[u8]) -> Result<Option<Arc<Class>>> {
        let arena = bumpalo::Bump::new();
        let classfile = Classfile::new(buff, &arena).map_err(|e| VmError::Other(e.to_string()))?;

        let name = classfile.class_name().ok_or_else(|| {
            RuntimeError::Execution("class file is missing its class name".into())
        })?;
        let (internal, external) = internal_and_external_names(name);

        let class = Class::from_classfile(classfile, &internal, external)?;

        Ok(Some(Arc::new(class)))
    }

    fn open_and_parse(&self, path: impl Into<PathBuf>) -> Result<Option<Arc<Class>>> {
        let mut file = File::open(path.into())
            .map(Some)
            .or_else(|e| match e.kind() {
                std::io::ErrorKind::NotFound => Ok::<_, VmError>(None),
                _ => Err(e.into()),
            })?;

        let Some(mut file) = file else {
            return Ok(None);
        };

        let mut buff = Vec::new();
        file.read_to_end(&mut buff)?;

        self.parse(&buff)
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
