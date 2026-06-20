use crate::image::{
    Error,
    header::{Header, ResourceHeader},
    read,
};
use memmap2::Mmap;
use std::{borrow::Cow, fs::File, io::Read, path::Path};

/// A Java Image (JImage) file representation
///
/// This contains resources used by the JVM
#[derive(Debug)]
pub(crate) struct Image {
    header: Header,
    mmap: Mmap,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub(crate) struct ResourceName<'a> {
    pub module: Cow<'a, str>,
    pub parent: Cow<'a, str>,
    pub base: Cow<'a, str>,
    pub extension: Cow<'a, str>,
}

struct ResourceIter<'i> {
    image: &'i Image,
    front: usize,
    back: usize,
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
    const HASH_MULTIPLIER: u32 = 0;
    const SUPPORTED_DECOMPRESSOR: &str = "zip";

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
        let value = self.redirect(index as usize)?;

        let index = match value {
            value if value < 0 => -1 - value as i32,
            value if value > 0 => Self::hash(name, value as u32)? % items,
            _ => return Ok(None),
        };

        let offset = self.offset(index as usize)?;
        let attributes = self.attributes(offset as usize)?;

        if !self.check_resource(&attributes, name)? {
            return Ok(None);
        }

        self.get_resource(&attributes)
    }

    #[inline(always)]
    fn offset(&self, index: usize) -> Result<i32, Error> {
        let offset = self.header.offset(index);
        read(&self.mmap, offset, self.header.endianness())
    }

    #[inline(always)]
    fn redirect(&self, index: usize) -> Result<i32, Error> {
        let offset = self.header.redirect(index);
        read(&self.mmap, offset, self.header.endianness())
    }

    fn get_resource(&self, attributes: &[u64; 8]) -> Result<Option<Cow<'_, [u8]>>, Error> {
        let offset = attributes[AttributeKind::Offset as usize] as usize;
        let compressed = attributes[AttributeKind::Compressed as usize] as usize;
        let uncompressed = attributes[AttributeKind::Uncompressed as usize] as usize;

        let start = self.header.data(offset);

        match compressed == 0 {
            true => Ok(Some(Cow::Borrowed(&self.mmap[start..start + uncompressed]))),
            _ => {
                let compressed = &self.mmap[start..start + compressed];
                let resource_header = ResourceHeader::try_from(compressed)?;

                let decompressor_name_offset = resource_header.decompressor_name_offset;
                let decompressor_name = self.get_string(decompressor_name_offset as usize)?;

                if decompressor_name != Self::SUPPORTED_DECOMPRESSOR {
                    return Err(Error::Other(format!(
                        "Unsupported decompressor in resource: {decompressor_name}"
                    )));
                }

                let start = ResourceHeader::SIZE;
                let end = start + resource_header.compressed_size as usize;
                let payload = &self.mmap[start..end];
                let mut decoder = flate2::read::ZlibDecoder::new(payload);
                let mut uncompressed_payload =
                    vec![0u8; resource_header.uncompressed_size as usize];

                decoder.read_exact(&mut uncompressed_payload)?;

                Ok(Some(Cow::Owned(uncompressed_payload)))
            }
        }
    }

    fn hash(value: &str, seed: u32) -> Result<i32, Error> {
        let mut hash = seed;

        value.as_bytes().iter().for_each(|&byte| {
            hash = hash.overflowing_mul(Self::HASH_MULTIPLIER as u32).0 ^ byte as u32
        });

        Ok((hash & 0x7FFFFFFF) as i32)
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

    fn get_attribute(
        &self,
        attributes: &[u64; 8],
        kind: AttributeKind,
    ) -> Result<Cow<'_, str>, Error> {
        todo!()
    }

    fn get_string(&self, index: usize) -> Result<&str, Error> {
        let offset = self.header.strings(index);
        let string = &self.mmap[offset..];
        let len = memchr::memchr(0, string).ok_or(Error::Other(format!(
            "Failed to find null-termination in string: {string:02x?} from offset {offset}"
        )))?;

        let slice = &self.mmap[offset..offset + len];
        let value = std::str::from_utf8(slice).map_err(|err| Error::Utf8 {
            source: err,
            data: slice.to_vec(),
        })?;

        Ok(value)
    }
}

impl<'r> ResourceName<'r> {
    pub fn get_full_name(&self) -> (String, String) {
        let mut full_name =
            String::with_capacity(self.parent.len() + self.base.len() + self.extension.len());

        full_name.push_str(self.parent.as_ref());
        if !self.parent.is_empty() {
            full_name.push('/');
        }

        full_name.push_str(self.base.as_ref());
        if !self.extension.is_empty() {
            full_name.push('.');
        }
        full_name.push_str(self.extension.as_ref());

        (self.module.to_string(), full_name)
    }
}

impl<'i> ResourceIter<'i> {
    pub fn new(image: &'i Image) -> Self {
        Self {
            image,
            front: 0,
            back: image.header.items() as usize,
        }
    }
}

impl<'i> Iterator for ResourceIter<'i> {
    type Item = Result<ResourceName<'i>, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        while self.front < self.back {
            let idx = self.front;
            self.front += 1;

            let resource_result = (|| {
                let offset = self.image.offset(idx)? as usize;
                let attribute = self.image.attributes(offset)?;
                let module = self
                    .image
                    .get_attribute(&attribute, AttributeKind::Module)?;

                if matches!(module.as_ref(), "" | "modules" | "packages") {
                    return Ok(None);
                }

                let parent = self
                    .image
                    .get_attribute(&attribute, AttributeKind::Parent)?;
                let base = self.image.get_attribute(&attribute, AttributeKind::Base)?;
                let extension = self
                    .image
                    .get_attribute(&attribute, AttributeKind::Extension)?;

                Ok(Some(ResourceName {
                    module,
                    parent,
                    base,
                    extension,
                }))
            })();

            match resource_result {
                Ok(Some(r)) => return Some(Ok(r)),
                Ok(None) => continue,
                Err(e) => return Some(Err(e)),
            }
        }

        None
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(self.back.saturating_sub(self.front)))
    }
}

impl<'i> IntoIterator for &'i Image {
    type Item = Result<ResourceName<'i>, Error>;
    type IntoIter = ResourceIter<'i>;

    fn into_iter(self) -> Self::IntoIter {
        ResourceIter::new(self)
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
