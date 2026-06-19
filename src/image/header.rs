//! Image File Header
//!
//! ## Header Fields
//!
//! ```text
//! ╭──────────────────┬────────┬──────┬──────────────────────────────────────╮
//! │ Field            │ Offset │ Size │ Description                          │
//! ├──────────────────┼────────┼──────┼──────────────────────────────────────┤
//! │ magic            │ 0      │ 4    │ 0xCAFEDADA identifying a Image file  │
//! │ version          │ 4      │ 4    │ Format version (major and minor)     │
//! │ flags            │ 8      │ 4    │ Reserved/flag                        │
//! │ resource count   │ 12     │ 4    │ Number of resources                  │
//! │ table length     │ 16     │ 4    │ Total byte length of all tables      │
//! │ locations size   │ 24     │ 4    │ File offset to the location table    │
//! │ strings size     │ 28     │ 4    │ File offset to the strings table     │
//! ╰──────────────────┴────────┴──────┴──────────────────────────────────────╯
//! ```

use crate::image::{Endianness, Error, read};
use byteorder::{BigEndian, ByteOrder, LittleEndian};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub(in crate::image) struct Header {
    version_major: u16,
    version_minor: u16,
    flags: u32,
    resource_count: u32,
    table_length: u32,
    locations_size: u32,
    strings_size: u32,
}

impl Header {
    const SIZE: usize = 28;
    /// respectively major and minor version supported
    const SUPPORTED_VERSIONS: (u16, u16) = (1, 0);

    pub(in crate::image) const fn bytes_length(&self) -> usize {
        self.table_length as usize * 4
    }
}

impl TryFrom<&[u8]> for Header {
    type Error = Error;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        let endianness = extract_endianness(bytes, super::MAGIC, "header")?;

        let mut position = 4;

        let version = read::<u32>(bytes, &mut position, endianness)?;
        let version_major = (version >> 16) as u16;
        let version_minor = (version & 0xFFFF) as u16;

        if (version_major, version_minor) != Self::SUPPORTED_VERSIONS {
            todo!()
        }

        let flags = read(bytes, &mut position, endianness)?;
        let resource_count = read(bytes, &mut position, endianness)?;
        let table_length = read(bytes, &mut position, endianness)?;
        let locations_size = read(bytes, &mut position, endianness)?;
        let strings_size = read(bytes, &mut position, endianness)?;

        Ok(Self {
            version_major,
            version_minor,
            flags,
            resource_count,
            table_length,
            locations_size,
            strings_size,
        })
    }
}

#[inline]
fn extract_endianness<'e>(bytes: &[u8], magic: u32, label: &'e str) -> Result<Endianness, Error> {
    let bytes = bytes.get(..4).ok_or(Error::BadRead { start: 0, end: 4 })?;

    if LittleEndian::read_u32(bytes) == magic {
        Ok(Endianness::Little)
    } else if BigEndian::read_u32(bytes) == magic {
        Ok(Endianness::Big)
    } else {
        Err(Error::Magic {
            magic: bytes.try_into().unwrap_or([0; 4]),
        })
    }
}
