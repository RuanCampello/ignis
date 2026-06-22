use crate::vm::{Result, VmError};
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
    legth: i32,
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
    Monotonic = 2,
    Variable = 3,
}

#[repr(u8)]
enum Flags {
    None = 0x0,
    Supported = 0x1,
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
}

impl Drop for PerfFile {
    fn drop(&mut self) {
        drop(self.mmap.take());
        let _ = std::fs::remove_file(&self.path);
    }
}

fn prologue() -> Result<Vec<u8>> {
    const PROLOGUE_SIZE: usize = 32;
    const HEADER_SIZE: usize = 20;

    let entries = 0i32;
    let actual_bytes = PROLOGUE_SIZE as i32;
    let offset = PROLOGUE_SIZE as i32;

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("timestamp to be valid")
        .as_nanos() as i64;
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
