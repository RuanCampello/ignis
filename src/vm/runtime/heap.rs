use crate::vm::{
    Result, VmError,
    runtime::{RuntimeError as Error, method_area::FieldValue},
};
use indexmap::IndexMap;
use once_cell::sync::Lazy;
use parking_lot::RwLock;
use std::sync::atomic::{AtomicI32, Ordering};

#[derive(Debug)]
pub(in crate::vm) struct Heap {
    /// Heap storage keyed by object reference id.
    objects: IndexMap<i32, HeapValue>,
}

static HEAP: Lazy<RwLock<Heap>> = Lazy::new(|| {
    RwLock::new(Heap {
        objects: IndexMap::new(),
    })
});

static HEAP_ID: AtomicI32 = AtomicI32::new(1);

#[derive(Debug)]
/// Represents a value on the heap.
enum HeapValue {
    Object(Instance),
    Array(Array),
}

#[derive(Debug)]
struct Array {
    name: String,
    value: Vec<u8>,
}

#[derive(Debug)]
/// Represents a Java object instance in the JVM heap.
pub(in crate::vm) struct Instance {
    /// Fully qualified class name of this object.
    name: String,
    /// Nested map of fields organized by class name and field name.
    fields: IndexMap<String, IndexMap<String, FieldValue>>,
}

pub(in crate::vm) fn with_heap<C, R>(callback: C) -> R
where
    C: FnOnce(&Heap) -> R,
{
    let heap = HEAP.read();
    callback(&heap)
}

pub(in crate::vm) fn with_mut_heap<C, R>(callback: C) -> R
where
    C: FnOnce(&mut Heap) -> R,
{
    let mut heap = HEAP.write();
    callback(&mut heap)
}

impl Heap {
    /// Allocates a new *zeroed* array in the heap with the given `length`.
    /// Returns its heap ID.
    pub fn allocate_array(&mut self, name: &str, length: i32) -> i32 {
        let element_size = Array::size(name);
        let len = (length as usize) * element_size;
        let value = vec![0u8; len];

        let array = Array {
            name: name.to_string(),
            value,
        };
        let id = Self::next_id();

        self.objects.insert(id, HeapValue::Array(array));
        id
    }

    // Allocates a new array in the heap initialised with the given values.
    // Returns its heap ID.
    pub fn allocate_array_with_values(&mut self, name: &str, array: Vec<u8>) -> i32 {
        let id = Self::next_id();
        let array = Array {
            name: name.to_string(),
            value: array,
        };

        self.objects.insert(id, HeapValue::Array(array));
        id
    }

    /// Allocates this given object instance into the heap.
    /// Returns its heap ID.
    pub fn allocate_instance(&mut self, instance: Instance) -> i32 {
        let id = Self::next_id();
        self.objects.insert(id, HeapValue::Object(instance));
        id
    }

    pub fn get_field_value<'a>(
        &'a self,
        obj_ref: i32,
        classname: &'a str,
        field: &'a str,
    ) -> Result<Vec<i32>> {
        if obj_ref == 0 {
            return Err(Error::InvalidObjectAcess {
                classname: classname.to_string(),
                field: field.to_string(),
            }
            .into());
        }

        match self.objects.get(&obj_ref) {
            Some(HeapValue::Object(instance)) => instance.get_value(classname, field),
            _ => Err(Error::InvalidObjectAcess {
                classname: classname.to_string(),
                field: field.to_string(),
            }
            .into()),
        }
    }

    pub fn get_array_value(&self, array_ref: i32, index: i32) -> Result<Vec<i32>> {
        match self.objects.get(&array_ref) {
            Some(HeapValue::Array(array)) => array.get(index),
            _ => Err(Error::InvalidArrayAccess(index as usize).into()),
        }
    }

    fn next_id() -> i32 {
        HEAP_ID.fetch_add(1, Ordering::Relaxed)
    }
}

impl Instance {
    fn get_value(&self, classname: &str, field: &str) -> Result<Vec<i32>> {
        self.lookup_field(classname, field)
            .and_then(|value| Some(value.value()))
            .ok_or(Error::InvalidObjectAcess {
                classname: classname.to_string(),
                field: field.to_string(),
            })?
    }

    fn lookup_field(&self, from: &str, field: &str) -> Option<&FieldValue> {
        match self.fields.get_index_of(from) {
            Some(index) => self
                .fields
                .iter()
                .take(index + 1)
                .rev()
                .find_map(|(_, map)| map.get(field)),
            _ => None,
        }
    }
}

impl Array {
    fn size(name: &str) -> usize {
        match name {
            "[B" => 1, // byte
            "[C" => 2, // char
            "[D" => 8, // double
            "[F" => 4, // float
            "[I" => 4, // int
            "[J" => 8, // long
            "[S" => 2, // short
            "[Z" => 1, // boolean
            _ => 4,    // object reference default
        }
    }

    fn get(&self, index: i32) -> Result<Vec<i32>> {
        let size = Self::size(&self.name);
        let offset = index as usize * size;

        let slice = &self.value[offset..offset + size];
        match size {
            1..4 => {
                let mut buff = [0u8; 4];
                match cfg!(target_endian = "big") {
                    true => buff[4 - size..4].copy_from_slice(slice),
                    false => buff[0..size].copy_from_slice(slice),
                };

                let value = i32::from_ne_bytes(buff);
                Ok(vec![value])
            }
            8 => {
                let mut buff = [0u8; 8];
                buff.copy_from_slice(slice);

                let hi = i32::from_ne_bytes(buff[0..4].try_into().unwrap());
                let lo = i32::from_ne_bytes(buff[4..8].try_into().unwrap());

                match cfg!(target_endian = "big") {
                    true => Ok(vec![hi, lo]),
                    false => Ok(vec![lo, hi]),
                }
            }
            _ => Err(Error::InvalidArrayEntrySize(size).into()),
        }
    }
}
