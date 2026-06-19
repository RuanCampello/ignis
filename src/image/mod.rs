//! This module is responsable for dealing with `JImage` files that are
//! used by the Java Plataform Module System

use std::f32::consts::E;

mod header;
mod image;

pub(in crate::image) const MAGIC: u32 = 0xCAFEDADA;

#[derive(Debug, Clone, Copy)]
pub(in crate::image) enum Endianness {
    Little,
    Big,
}

pub(self) trait FromBytes: Sized {
    fn from_le(bytes: &[u8]) -> Self;
    fn from_be(bytes: &[u8]) -> Self;
}

macro_rules! impl_from_bytes {
    ($($t:ty),*) => {
        $(impl FromBytes for $t {
            fn from_le(bytes: &[u8]) -> Self { Self::from_le_bytes(bytes.try_into().unwrap()) }
            fn from_be(bytes: &[u8]) -> Self { Self::from_be_bytes(bytes.try_into().unwrap()) }
        })*
    };
}
impl_from_bytes!(u16, u32, u64, i16, i32, i64);

fn read<T: FromBytes>(bytes: &[u8], offset: &mut usize, endianness: Endianness) -> Result<T, ()> {
    let start = *offset;
    let end = start + std::mem::size_of::<T>();

    let slice = bytes.get(start..end).ok_or(())?;

    *offset = end;

    match endianness {
        Endianness::Little => Ok(T::from_le(slice)),
        Endianness::Big => Ok(T::from_be(slice)),
    }
}
