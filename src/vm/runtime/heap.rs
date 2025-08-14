use crate::vm::runtime::method_area::FieldValue;
use indexmap::IndexMap;
use std::sync::atomic::{AtomicI32, Ordering};

#[derive(Debug)]
pub(in crate::vm::runtime) struct Heap<'h> {
    /// Memory arena used for allocating heap values such as arrays.
    /// Currently, the heap lives for the duration of the program, so all arena allocations
    /// remain valid until the heap is dropped.
    ///
    /// *Note*: when implementing garbage collection, this allocation strategy may need to change
    /// to support moving or freeing individual objects.
    arena: bumpalo::Bump,

    /// Heap storage keyed by object reference id.
    objects: IndexMap<i32, HeapValue<'h>>,
}

static HEAP_ID: AtomicI32 = AtomicI32::new(1);

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Debug)]
/// Represents a value on the heap.
enum HeapValue<'h> {
    Object(Instance),
    Array(Array<'h>),
}

#[derive(Debug)]
struct Array<'h> {
    name: &'h str,
    value: &'h [u8],
}

#[derive(Debug)]
/// Represents a Java object instance in the JVM heap.
struct Instance {
    /// Fully qualified class name of this object.
    name: String,
    /// Nested map of fields organized by class name and field name.
    fields: IndexMap<String, IndexMap<String, FieldValue>>,
}

impl<'h> Heap<'h> {
    /// Allocates a new *zeroed* array in the heap with the given `length`.
    /// Returns its heap ID.
    pub fn allocate_array(&'h mut self, name: &'h str, length: i32) -> i32 {
        let element_size = Array::size(name);
        let len = (length as usize) * element_size;
        let value = self.arena.alloc_slice_fill_copy(len, 0u8);

        let array = Array { name, value };
        let id = Self::next_id();

        self.objects.insert(id, HeapValue::Array(array));
        id
    }

    // Allocates a new array in the heap initialised with the given values.
    // Returns its heap ID.
    pub fn allocate_array_with_values(&'h mut self, name: &'h str, array: &'h [u8]) -> i32 {
        let id = Self::next_id();
        let array = Array {
            name,
            value: self.arena.alloc_slice_copy(array),
        };

        self.objects.insert(id, HeapValue::Array(array));
        id
    }

    pub fn next_id() -> i32 {
        HEAP_ID.fetch_add(1, Ordering::Relaxed)
    }
}

impl<'a> Array<'a> {
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
