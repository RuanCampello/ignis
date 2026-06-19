//! This module is responsable for dealing with `JImage` files that are
//! used by the Java Plataform Module System

mod header;
mod image;

pub(in crate::image) const MAGIC: u32 = 0xCAFEDADA;

#[derive(Debug, thiserror::Error)]
pub(in crate::image) enum Error {
    #[error("File doesn't match a valid jimage. Found {magic:02x?}")]
    Magic { magic: [u8; 4] },
    #[error("Invalid jimage version: {version_major}.{version_minor}")]
    InvalidVersion {
        version_major: u16,
        version_minor: u16,
    },
    #[error("Unable to read from slice: [{start}..{end}]")]
    BadRead { start: usize, end: usize },
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Internal error: {0}")]
    Other(String),
}

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

#[inline]
fn read<T: FromBytes>(
    bytes: &[u8],
    offset: &mut usize,
    endianness: Endianness,
) -> Result<T, Error> {
    let start = *offset;
    let end = start + std::mem::size_of::<T>();

    let slice = bytes.get(start..end).ok_or(Error::BadRead { start, end })?;

    *offset = end;

    match endianness {
        Endianness::Little => Ok(T::from_le(slice)),
        Endianness::Big => Ok(T::from_be(slice)),
    }
}
