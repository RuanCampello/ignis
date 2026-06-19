use std::{fs::File, path::Path};

use memmap2::Mmap;

use crate::image::{Error, header::Header};

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
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        let file = File::open(path.as_ref())?;
        let mmap = unsafe { Mmap::map(&file)? };
        let header = Header::try_from(mmap.as_ref())?;

        Ok(Self { header, mmap })
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
