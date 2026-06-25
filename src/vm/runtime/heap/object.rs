use dashmap::DashMap;
use dashmap::mapref::one::{Ref, RefMut};
use std::sync::atomic::{AtomicI32, Ordering};

#[derive(Debug)]
pub(in crate::vm::runtime::heap) struct Objects<V> {
    map: DashMap<i32, V>,
    counter: AtomicI32,
}

impl<V> Objects<V> {
    pub fn new() -> Self {
        Self {
            map: DashMap::new(),
            counter: AtomicI32::new(0),
        }
    }

    pub fn insert(&self, value: V) -> i32 {
        let id = self.counter.fetch_add(1, Ordering::Relaxed) + 1;
        self.map.insert(id, value);
        id
    }

    pub fn get(&self, key: &i32) -> Option<Ref<'_, i32, V>> {
        self.map.get(key)
    }

    pub fn get_mut(&self, key: &i32) -> Option<RefMut<'_, i32, V>> {
        self.map.get_mut(key)
    }
}
