use crate::{
    Args,
    vm::{Result, VmError, runtime::RuntimeError},
};
use memmap2::MmapMut;
use once_cell::sync::{Lazy, OnceCell};
use parking_lot::Mutex;
use std::{collections::HashSet, fs::OpenOptions, path::PathBuf};

struct PerfFile {
    mmap: Option<MmapMut>,
    path: PathBuf,
    names: HashSet<String>,
}

/// Strongly based on the [PerfDataEntry] of openjdk's [PerfDataBuffer]
///
/// [PerfDataBuffer]: https://openjdk.org/groups/serviceability/jvmstat/sun/jvmstat/perfdata/monitor/v2_0/PerfDataBuffer.html
/// [PerfDataEntry]: https://github.com/openjdk/jdk/blob/3a3206b8f272dcd52bba05d4ad2079c223826c23/src/hotspot/share/runtime/perfMemory.hpp#L80-L102
struct Entry {
    name: String,
    typ: u8,
    flags: u8,
    units: u8,
    var: u8,
    data: Vec<u8>,
    length: i32,
}

#[repr(u8)]
enum Types {
    Byte = b'B',
    Boolean = b'Z',
    Long = b'J',
    Int = b'I',
    Short = b'S',
    Char = b'C',
    Double = b'D',
    Float = b'F',
    Void = b'V',
    Reference = b'L',
    Array = b'[',
}

#[repr(u8)]
enum Var {
    Constant = 1,
    Monotonic,
    Variable,
}

#[repr(u8)]
enum Flags {
    None = 0x0,
    Supported = 0x1,
}

#[repr(u8)]
enum Units {
    None = 1,
    Bytes,
    Ticks,
    Events,
    String,
    Hertz,
}

static PERF_FILE: OnceCell<Mutex<PerfFile>> = OnceCell::new();
static PAGE_CAPACITY: Lazy<usize> = Lazy::new(|| {
    const SIZE: usize = 1024 << 5;
    let page_size = page_size::get();

    match SIZE % page_size == 0 {
        true => SIZE,
        _ => SIZE + page_size - (SIZE % page_size),
    }
});

const IS_LITTLE_ENDIAN: bool = cfg!(target_endian = "little");

const MAGIC: i32 = match IS_LITTLE_ENDIAN {
    true => 0xc0c0fecau32 as i32,
    _ => 0xcafec0c0u32 as i32,
};

const BYTE_ORDER: u8 = if IS_LITTLE_ENDIAN { 1 } else { 0 };

impl<'a> Args<'a> {
    pub(in crate::vm) fn initialise_perf_file(&self) -> Result<()> {
        let mut perf_file = PERF_FILE
            .get_or_try_init(|| Ok::<Mutex<PerfFile>, VmError>(Mutex::new(PerfFile::default()?)))?
            .lock();

        let command = {
            let mut cmd = self.entry.to_string();
            for arg in &self.program_args {
                cmd.push(' ');
                cmd.push_str(arg);
            }

            cmd
        };

        perf_file.add_string("sun.rt.javaCommand", &command)?;

        Ok(())
    }
}

impl PerfFile {
    pub(in crate::vm) fn default() -> Result<Self> {
        let pid = std::process::id();
        let perf_dir = get_dir()?;

        std::fs::create_dir_all(&perf_dir)?;

        let file_path = perf_dir.join(pid.to_string());

        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(&file_path)?;
        file.set_len(*PAGE_CAPACITY as u64)?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            file.set_permissions(std::fs::Permissions::from_mode(0o600))?;
        }

        // SAFETY: the file to map was created here
        // so there's no other procress mapping it yet
        let mut mmap = unsafe { MmapMut::map_mut(&file)? };

        let prologue = prologue()?;
        mmap[..prologue.len()].copy_from_slice(&prologue);
        mmap.flush()?;

        Ok(Self {
            mmap: Some(mmap),
            path: file_path,
            names: HashSet::new(),
        })
    }

    #[inline]
    fn add_string(&mut self, name: &str, value: &str) -> Result<(*const u8, usize)> {
        self.add_array(
            name,
            Var::Constant as u8,
            Units::String as u8,
            value.as_bytes(),
            1024,
        )
    }

    fn add_array(
        &mut self,
        name: &str,
        var: u8,
        units: u8,
        value: &[u8],
        len: usize,
    ) -> Result<(*const u8, usize)> {
        let len = len.max(1);
        let mut data = vec![0u8; len];
        let copy_len = value.len().min(len.saturating_sub(1));
        data[..copy_len].copy_from_slice(&value[..copy_len]);

        let entry = Entry {
            units,
            var,
            data,
            typ: Types::Byte as u8,
            flags: flags(name),
            name: name.to_string(),
            length: len as i32,
        };

        self.append(entry)
    }

    fn add_long(
        &mut self,
        name: &str,
        var: u8,
        units: u8,
        value: i64,
    ) -> Result<(*const u8, usize)> {
        let entry = Entry {
            name: name.to_string(),
            typ: Types::Long as u8,
            flags: flags(name),
            units,
            var,
            data: value.to_ne_bytes().to_vec(),
            length: 0,
        };

        self.append(entry)
    }

    fn append(&mut self, entry: Entry) -> Result<(*const u8, usize)> {
        let Some(mmap) = self.mmap.as_mut() else {
            return Err(RuntimeError::Execution("perf_file mmap is not available".into()).into());
        };

        let used = i32::from_ne_bytes(mmap[8..12].try_into().map_err(|e| {
            VmError::Other(format!(
                "failed to read the perf_file on {:02x?}",
                &mmap[8..12]
            ))
        })?) as usize;
        let num_entries = (i32::from_ne_bytes(mmap[28..32].try_into().map_err(|e| {
            VmError::Other(format!(
                "failed to read the perf_file on {:02x?}",
                &mmap[28..32]
            ))
        })?) + 1) as usize;

        let (bytes, offset, end) = entry.to_bytes();

        let new_used = used + bytes.len();
        let offset = offset + used;
        let end = end + used;

        if new_used > *PAGE_CAPACITY {
            mmap[12..16].copy_from_slice(&(new_used as i32 - *PAGE_CAPACITY as i32).to_ne_bytes());
            return Err(VmError::Other(
                "Not enough space in perf data file for new entry to be appended".into(),
            ));
        }

        mmap[used..new_used].copy_from_slice(&bytes);
        mmap[8..12].copy_from_slice(&(new_used as i32).to_ne_bytes());
        mmap[16..24].copy_from_slice(&timestamp().to_ne_bytes());
        mmap[28..32].copy_from_slice(&(num_entries).to_ne_bytes());
        mmap.flush()?;

        self.names.insert(entry.name);
        Ok((mmap[offset..end].as_ptr(), end - offset))
    }

    #[inline(always)]
    fn contains(&self, name: &str) -> bool {
        self.names.contains(name)
    }
}

impl Entry {
    const HEADER_SIZE: usize = 20;

    fn to_bytes(&self) -> (Vec<u8>, usize, usize) {
        fn align(value: usize, align: usize) -> usize {
            (value + align - 1) & !(align - 1)
        }

        let name = Self::HEADER_SIZE as i32;
        let name_len = self.name.len() + 1;
        let name_end = Self::HEADER_SIZE + name_len;

        let offset = align(name_end, 8);
        let end = offset + self.data.len();
        let length = align(end, 8);

        let mut buff = Vec::with_capacity(length);

        buff.extend_from_slice(&(length as i32).to_ne_bytes());
        buff.extend_from_slice(&offset.to_ne_bytes());
        buff.extend_from_slice(&self.length.to_ne_bytes());
        buff.push(self.typ);
        buff.push(self.flags);
        buff.push(self.units);
        buff.push(self.var);
        buff.extend_from_slice(&(offset as i32).to_ne_bytes());

        debug_assert_eq!(buff.len(), Self::HEADER_SIZE);

        buff.extend_from_slice(self.name.as_bytes());
        buff.push(0u8);
        buff.resize(offset, 0u8);
        buff.extend_from_slice(&self.data);
        buff.resize(length, 0u8);

        (buff, offset, length)
    }
}

impl Drop for PerfFile {
    fn drop(&mut self) {
        drop(self.mmap.take());
        let _ = std::fs::remove_file(&self.path);
    }
}

fn prologue() -> Result<Vec<u8>> {
    const PROLOGUE_SIZE: usize = 32;

    let entries = 0i32;
    let actual_bytes = PROLOGUE_SIZE as i32;
    let offset = PROLOGUE_SIZE as i32;

    let timestamp = timestamp();
    let accessible = 1;
    let overflow = 0i32;

    let mut buff = Vec::with_capacity(PROLOGUE_SIZE);
    buff.extend_from_slice(&MAGIC.to_ne_bytes());
    buff.push(BYTE_ORDER);
    buff.push(2);
    buff.push(0);
    buff.push(accessible);
    buff.extend_from_slice(&actual_bytes.to_ne_bytes());
    buff.extend_from_slice(&overflow.to_ne_bytes());
    buff.extend_from_slice(&timestamp.to_ne_bytes());
    buff.extend_from_slice(&offset.to_ne_bytes());
    buff.extend_from_slice(&entries.to_ne_bytes());

    debug_assert_eq!(buff.len(), PROLOGUE_SIZE);
    Ok(buff)
}

fn get_dir() -> Result<PathBuf> {
    use std::env;

    let temp_dir = env::temp_dir();

    let username = match whoami::username() {
        Ok(username) => username,
        _ => env::var("USER")
            .or_else(|_| env::var("LOGNAME"))
            .map_err(|e| VmError::Other(format!("Couldn't access the username variable: {e}")))?,
    };

    let username: String = username
        .chars()
        .map(|c| match c.is_alphabetic() || c == '_' || c == '-' {
            true => c,
            _ => '_',
        })
        .collect();

    let path = temp_dir.join(format!("hsperfdata_{username}"));

    Ok(path)
}

fn timestamp() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("timestamp to be valid")
        .as_nanos() as i64
}

#[inline(always)]
fn flags(name: &str) -> u8 {
    return match name.starts_with("java.") || name.starts_with("com.sun.") {
        true => Flags::Supported,
        _ => Flags::None,
    } as u8;
}
