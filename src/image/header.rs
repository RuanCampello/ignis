//! Image file and resource header
use crate::image::{Endianness, Error, read_mut};
use byteorder::{BigEndian, ByteOrder, LittleEndian};

/// `JImage` file header
///
/// ```text
/// ╭──────────────────┬────────┬──────┬──────────────────────────────────────╮
/// │ Field            │ Offset │ Size │ Description                          │
/// ├──────────────────┼────────┼──────┼──────────────────────────────────────┤
/// │ magic            │ 0      │ 4    │ 0xCAFEDADA identifying a Image file  │
/// │ version          │ 4      │ 4    │ Format version (major and minor)     │
/// │ flags            │ 8      │ 4    │ Reserved/flag                        │
/// │ resource count   │ 12     │ 4    │ Number of resources                  │
/// │ table length     │ 16     │ 4    │ Total byte length of all tables      │
/// │ locations size   │ 24     │ 4    │ File offset to the location table    │
/// │ strings size     │ 28     │ 4    │ File offset to the strings table     │
/// ╰──────────────────┴────────┴──────┴──────────────────────────────────────╯
/// ```
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[repr(C)]
pub(in crate::image) struct Header {
    version_major: u16,
    version_minor: u16,
    flags: u32,
    resource_count: u32,
    table_length: u32,
    locations_size: u32,
    strings_size: u32,
    endianness: Endianness,
}

#[derive(Debug, Clone, Copy)]
pub(in crate::image) struct ResourceHeader {
    pub(in crate::image) compressed_size: u64,
    pub(in crate::image) uncompressed_size: u64,
    pub(in crate::image) decompressor_name_offset: u32,
    decompressor_config_offset: u32,

    is_terminal: u8,
}

impl Header {
    const SIZE: usize = 28;
    /// respectively major and minor version supported
    const SUPPORTED_VERSIONS: (u16, u16) = (1, 0);

    pub(in crate::image) const fn bytes_length(&self) -> usize {
        self.table_length as usize * 4
    }

    pub(in crate::image) const fn items(&self) -> u32 {
        self.table_length
    }

    pub(in crate::image) const fn redirect(&self, position: usize) -> usize {
        Self::SIZE + position * 4
    }

    pub(in crate::image) const fn endianness(&self) -> Endianness {
        self.endianness
    }

    pub(in crate::image) const fn offset(&self, position: usize) -> usize {
        Self::SIZE + self.bytes_length() + position * 4
    }

    pub(in crate::image) const fn attributes(&self, position: usize) -> usize {
        self.attributes_begins_at() + position
    }

    pub(in crate::image) const fn strings(&self, position: usize) -> usize {
        self.attributes_begins_at() + self.locations_size as usize + position
    }

    pub(in crate::image) const fn data(&self, position: usize) -> usize {
        self.attributes_begins_at()
            + self.locations_size as usize
            + self.strings_size as usize
            + position
    }

    const fn attributes_begins_at(&self) -> usize {
        Self::SIZE + self.bytes_length() * 2
    }
}

impl TryFrom<&[u8]> for Header {
    type Error = Error;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        let endianness = extract_endianness(bytes, super::MAGIC)?;

        let mut position = 4;

        let version = read_mut::<u32>(bytes, &mut position, endianness)?;
        let version_major = (version >> 16) as u16;
        let version_minor = (version & 0xFFFF) as u16;

        if (version_major, version_minor) != Self::SUPPORTED_VERSIONS {
            return Err(Error::InvalidVersion {
                version_major,
                version_minor,
            });
        }

        let flags = read_mut(bytes, &mut position, endianness)?;
        let resource_count = read_mut(bytes, &mut position, endianness)?;
        let table_length = read_mut(bytes, &mut position, endianness)?;
        let locations_size = read_mut(bytes, &mut position, endianness)?;
        let strings_size = read_mut(bytes, &mut position, endianness)?;

        Ok(Self {
            version_major,
            version_minor,
            flags,
            resource_count,
            table_length,
            locations_size,
            strings_size,
            endianness,
        })
    }
}

impl ResourceHeader {
    pub(in crate::image) const SIZE: usize = 29;
}

impl TryFrom<&[u8]> for ResourceHeader {
    type Error = Error;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let endianness = extract_endianness(value, 0xCAFEFAFA)?;

        let mut position = 4;

        let compressed_size = read_mut(value, &mut position, endianness)?;
        let uncompressed_size = read_mut(value, &mut position, endianness)?;
        let decompressor_name_offset = read_mut(value, &mut position, endianness)?;
        let decompressor_config_offset = read_mut(value, &mut position, endianness)?;
        let is_terminal = value[position];

        Ok(Self {
            compressed_size,
            uncompressed_size,
            decompressor_name_offset,
            decompressor_config_offset,
            is_terminal,
        })
    }
}

#[inline]
fn extract_endianness<'e>(bytes: &[u8], magic: u32) -> Result<Endianness, Error> {
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
