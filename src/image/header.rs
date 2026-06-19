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

use crate::image::Endianness;
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
}

impl TryFrom<&[u8]> for Header {
    type Error = ();

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        let endianness = extract_endianness(bytes, super::MAGIC, "header")?;

        let mut position = 4;

        todo!()
    }
}

#[inline]
fn extract_endianness<'e>(bytes: &[u8], magic: u32, label: &'e str) -> Result<Endianness, ()> {
    let bytes = bytes.get(..4).ok_or(())?;

    if LittleEndian::read_u32(bytes) == magic {
        Ok(Endianness::Little)
    } else if BigEndian::read_u32(bytes) == magic {
        Ok(Endianness::Big)
    } else {
        Err(())
    }
}
