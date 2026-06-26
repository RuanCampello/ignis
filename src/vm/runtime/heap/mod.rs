use crate::vm::{
    Result, VmError,
    runtime::{RuntimeError as Error, heap::object::Objects, method_area::class::FieldValue},
};
use dashmap::DashMap;
use indexmap::IndexMap;
use once_cell::sync::Lazy;
use parking_lot::RwLock;
use std::sync::{
    Arc, LazyLock,
    atomic::{AtomicI32, Ordering},
};

mod object;

#[derive(Debug)]
pub(in crate::vm) struct Heap {
    /// Heap storage keyed by object reference id.
    objects: Objects<HeapValue>,
    ref_by_string: DashMap<String, i32>,
}

pub(in crate::vm) static HEAP: LazyLock<Heap> = LazyLock::new(Heap::default);

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

#[derive(Debug, Clone)]
pub(in crate::vm) enum Instance {
    Base(BaseInstance),
    Class(ClassInstance),
}

#[derive(Debug, Clone)]
/// Represents a Java object instance in the JVM heap.
pub(in crate::vm) struct BaseInstance {
    pub id: usize,
    /// Nested map of fields organized by class name and field name.
    pub fields: IndexMap<String, IndexMap<String, FieldValue>>,
}

#[derive(Debug, Clone)]
pub(in crate::vm) struct ClassInstance {
    pub(in crate::vm::runtime) instance: BaseInstance,
    pub(in crate::vm::runtime) class_id: usize,
}

impl Heap {
    /// Allocates a new *zeroed* array in the heap with the given `length`.
    /// Returns its heap ID.
    pub fn allocate_array(&self, name: &str, length: i32) -> i32 {
        let element_size = Array::size(name);
        let len = (length as usize) * element_size;
        let value = vec![0u8; len];

        let array = Array {
            name: name.to_string(),
            value,
        };

        self.objects.insert(HeapValue::Array(array))
    }

    // Allocates a new array in the heap initialised with the given raw bytes.
    // Returns its heap ID.
    pub fn allocate_array_with_values(&self, name: &str, array: Vec<u8>) -> i32 {
        let array = Array {
            name: name.to_string(),
            value: array,
        };

        self.objects.insert(HeapValue::Array(array))
    }

    pub(in crate::vm::runtime) fn allocate_int_array(&self, values: &[i32]) -> i32 {
        let bytes = values
            .iter()
            .flat_map(|value| value.to_ne_bytes())
            .collect();
        self.allocate_array_with_values("[I", bytes)
    }

    pub(in crate::vm::runtime) fn interned_string(&self, value: &str) -> Option<i32> {
        self.ref_by_string.get(value).map(|entry| *entry.value())
    }

    pub(in crate::vm::runtime) fn intern_string(&self, value: &str, reference: i32) {
        self.ref_by_string.insert(value.to_string(), reference);
    }

    pub(in crate::vm::runtime) fn get_mirror_class_id(&self, class_ref: i32) -> Result<usize> {
        self.objects
            .get(&class_ref)
            .and_then(|entry| match entry.value() {
                HeapValue::Object(Instance::Class(class_instance)) => Some(class_instance.class_id),
                // HeapValue::Object(Instance::Base(instance)) => Some(instance.id),
                _ => None,
            })
            .ok_or_else(|| {
                Error::Execution(format!("failed to get mirror class id by ref: {class_ref}"))
                    .into()
            })
    }

    /// Allocates this given object instance into the heap.
    /// Returns its heap ID.
    pub fn allocate_instance(&self, instance: Instance) -> i32 {
        self.objects.insert(HeapValue::Object(instance))
    }

    pub fn get_field_value<'a>(
        &'a self,
        obj_ref: i32,
        classname: &'a str,
        field: &'a str,
    ) -> Result<Vec<i32>> {
        let error = || Error::InvalidObjectAcess {
            classname: classname.to_string(),
            field: field.to_string(),
        };

        if obj_ref == 0 {
            return Err(error().into());
        }

        let entry = self.objects.get(&obj_ref).ok_or_else(|| error())?;
        match entry.value() {
            HeapValue::Object(instance) => instance.get_value(classname, field),
            _ => Err(error().into()),
        }
    }

    /// raw backing bytes of the array at `array_ref`, exactly as stored
    pub(in crate::vm::runtime) fn array_bytes(&self, array_ref: i32) -> Result<Vec<u8>> {
        let entry = self.objects.get(&array_ref).ok_or_else(|| {
            Error::Execution(format!("array reference was not found: {array_ref}"))
        })?;

        match entry.value() {
            HeapValue::Array(array) => Ok(array.value.clone()),
            _ => Err(Error::Execution(format!("reference {array_ref} is not an array")).into()),
        }
    }

    pub fn get_array_value(&self, array_ref: i32, index: i32) -> Result<Vec<i32>> {
        let entry = self
            .objects
            .get(&array_ref)
            .ok_or_else(|| Error::InvalidArrayEntrySize(index as usize))?;

        match entry.value() {
            HeapValue::Array(array) => {
                let value = array.get(index)?;
                Ok(value)
            }
            _ => Err(Error::InvalidArrayEntrySize(index as usize).into()),
        }
    }

    pub fn set_array_value(&self, array_ref: i32, index: i32, value: Vec<i32>) -> Result<()> {
        let mut entry = self
            .objects
            .get_mut(&array_ref)
            .ok_or_else(|| Error::InvalidArrayAccess(index as usize))?;

        match entry.value_mut() {
            HeapValue::Array(array) => array.set(index, value),
            _ => Err(Error::InvalidArrayAccess(index as usize).into()),
        }
    }
}

impl Default for Heap {
    fn default() -> Self {
        Self {
            objects: Objects::new(),
            ref_by_string: DashMap::new(),
        }
    }
}

impl Instance {
    pub fn set_field_value(
        &mut self,
        class_name: &str,
        field_name_type: &str,
        value: Vec<i32>,
    ) -> Result<()> {
        self.lookup_mut_field(class_name, field_name_type)
            .map(|v| v.set(value))
            .ok_or_else(|| {
                Error::Execution(format!(
                    "error setting value for instance: {class_name}.{field_name_type}"
                ))
            })?
    }

    fn lookup_mut_field(
        &mut self,
        starting: &str,
        field_name_type: &str,
    ) -> Option<&mut FieldValue> {
        let instance = match self {
            Self::Base(instance) => instance,
            Self::Class(class) => &mut class.instance,
        };

        instance.fields.get_index_of(starting).and_then(|index| {
            instance
                .fields
                .iter_mut()
                .take(index + 1)
                .rev()
                .find_map(|(_, map)| map.get_mut(field_name_type))
        })
    }

    fn get_value(&self, classname: &str, field: &str) -> Result<Vec<i32>> {
        self.lookup_field(classname, field)
            .and_then(|value| Some(value.value()))
            .ok_or(Error::InvalidObjectAcess {
                classname: classname.to_string(),
                field: field.to_string(),
            })?
    }

    fn lookup_field(&self, from: &str, field: &str) -> Option<&FieldValue> {
        let instance = match self {
            Self::Base(instance) => instance,
            Self::Class(class) => &class.instance,
        };

        match instance.fields.get_index_of(from) {
            Some(index) => instance
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

    /// writes `value` into the slot at `index`, encoding it the inverse way [Array::get]
    /// reads it back
    fn set(&mut self, index: i32, value: Vec<i32>) -> Result<()> {
        let size = Self::size(&self.name);
        let offset = index as usize * size;
        let slot = &mut self.value[offset..offset + size];

        match size {
            1..=4 => {
                let bytes = value[0].to_ne_bytes();
                match cfg!(target_endian = "big") {
                    true => slot.copy_from_slice(&bytes[4 - size..4]),
                    false => slot.copy_from_slice(&bytes[0..size]),
                }
            }
            8 => {
                let (hi, lo) = match cfg!(target_endian = "big") {
                    true => (value[0], value[1]),
                    false => (value[1], value[0]),
                };

                slot[0..4].copy_from_slice(&hi.to_ne_bytes());
                slot[4..8].copy_from_slice(&lo.to_ne_bytes());
            }
            _ => return Err(Error::InvalidArrayEntrySize(size).into()),
        }

        Ok(())
    }
}
