use indexmap::IndexMap;

#[derive(Debug)]
pub(in crate::vm::runtime) struct Heap<'h> {
    /// Heap storage keyed by object reference id.
    objects: IndexMap<i32, HeapValue<'h>>,
}

#[derive(Debug)]
enum HeapValue<'h> {
    Object,
    Array { name: &'h str, value: &'h [u8] },
}
