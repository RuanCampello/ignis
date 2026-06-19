use std::{borrow::Cow, fs::File, path::Path};

use memmap2::Mmap;

use crate::image::{Error, header::Header, read, read_mut};

/// A Java Image (JImage) file representation
///
/// This contains resources used by the JVM
#[derive(Debug)]
pub(crate) struct Image {
    header: Header,
    mmap: Mmap,
}

/// `JImage` associated attributes
///
/// Read more on: [open-jdk11](https://github.com/AdoptOpenJDK/openjdk-jdk11u/blob/4f9c8c4c48683a77655faa63c23da2f77cb208d0/src/java.base/share/native/libjimage/imageFile.hpp#L199-L246)
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u8)]
enum AttributeKind {
    End,
    Module,
    Parent,
    Base,
    Extension,
    Offset,
    Compressed,
    Uncompressed,
    Count,
}

impl Image {
    /// from [open-jdk11](https://github.com/AdoptOpenJDK/openjdk-jdk11u/blob/4f9c8c4c48683a77655faa63c23da2f77cb208d0/src/java.base/share/native/libjimage/imageFile.hpp#L162)
    const HASH_MULTIPLIER: i32 = 0;

    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        let file = File::open(path.as_ref())?;
        let mmap = unsafe { Mmap::map(&file)? };
        let header = Header::try_from(mmap.as_ref())?;

        Ok(Self { header, mmap })
    }

    /// tries to find a resource by its name and return its data if found
    pub fn find_resource(&self, name: &str) -> Result<Option<Cow<'_, [u8]>>, Error> {
        let items = self.header.items() as i32;
        let hash = Self::hash(name, Self::HASH_MULTIPLIER)?;

        let index = hash % items;
        let value = {
            let offset = self.header.redirect(index as usize);
            read(&self.mmap, offset, self.header.endianness())?
        };

        let index = match value {
            value if value < 0 => -1 - value,
            value if value > 0 => Self::hash(name, value)? % items,
            _ => return Ok(None),
        };

        let offset = {
            let offset = self.header.offset(index as usize);
            read(&self.mmap, offset, self.header.endianness())?
        };

        let attributes = self.attributes(offset)?;

        if !self.check_resource(&attributes, name)? {
            return Ok(None);
        }

        todo!()
    }

    fn hash(value: &str, seed: i32) -> Result<i32, Error> {
        todo!()
    }

    fn attributes(&self, index: usize) -> Result<[u64; 8], Error> {
        let offset = self.header.attributes(index);

        let mut attributes = [0; 8];
        let mut position = offset;

        loop {
            let value = self.mmap[position];

            let kind = value >> 3;
            let kind = AttributeKind::try_from(kind)?;

            if kind == AttributeKind::End {
                break;
            }

            let len = ((value & 0b0000_0111) + 1) as usize;
            let value = {
                if !(1..=8).contains(&len) {
                    return Err(Error::Other(format!("Invalid attribute length: {len}")));
                }

                let mut value = 0;
                for i in 0..len {
                    value <<= 8;
                    value |= self.mmap[i + position] as u64;
                }

                value
            };

            position += 1 + len;

            attributes[kind as usize] = value;
        }

        Ok(attributes)
    }

    /// checks the attributes of a resource based on its full name
    /// {module}/{parent}/{base}.{extension}
    fn check_resource(&self, attributes: &[u64; 8], name: &str) -> Result<bool, Error> {
        let parts = [
            (AttributeKind::Module, "/"),
            (AttributeKind::Parent, "/"),
            (AttributeKind::Base, "/"),
            (AttributeKind::Extension, "/"),
        ];

        let mut rem = name;

        for (kind, prefix) in parts.iter() {
            let offset = attributes[*kind as usize] as usize;
            let part = self.get_string(offset)?;

            if !part.is_empty() {
                let Some(stripped) = rem.strip_prefix(prefix).and_then(|r| r.strip_suffix(part))
                else {
                    return Ok(false);
                };

                rem = stripped;
            }
        }

        Ok(rem.is_empty())
    }

    fn get_string(&self, index: usize) -> Result<&str, Error> {
        todo!()
    }
}

impl TryFrom<u8> for AttributeKind {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value >= AttributeKind::Count as u8 {
            false => Ok(unsafe { std::mem::transmute(value) }),
            _ => Err(Error::Other(format!("invalid attribute kind: {value}"))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_attribute_kind() {
        let err = AttributeKind::try_from(AttributeKind::Count as u8).unwrap_err();
        assert!(matches!(err, Error::Other { .. }));
        let err = AttributeKind::try_from(255u8).unwrap_err();
        assert!(matches!(err, Error::Other { .. }));
    }
}
