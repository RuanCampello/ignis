use crate::vm::runtime::method_area::FieldValue;
use indexmap::IndexMap;
use once_cell::sync::Lazy;
use parking_lot::RwLock;
use std::sync::atomic::{AtomicI32, Ordering};

#[derive(Debug)]
pub(in crate::vm::runtime) struct Heap {
    /// Heap storage keyed by object reference id.
    objects: IndexMap<i32, HeapValue>,
}

static HEAP: Lazy<RwLock<Heap>> = Lazy::new(|| {
    RwLock::new(Heap {
        objects: IndexMap::new(),
    })
});

static HEAP_ID: AtomicI32 = AtomicI32::new(1);

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

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
struct Instance {
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

    pub fn next_id() -> i32 {
        HEAP_ID.fetch_add(1, Ordering::Relaxed)
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
}
