#[inline]
pub const fn path_separator<'s>() -> &'s str {
    match cfg!(target_os = "windows") {
        true => ";",
        false => ":",
    }
}

#[inline]
pub const fn file_separator<'s>() -> &'s str {
    match cfg!(target_os = "windows") {
        true => "\\",
        false => "/",
    }
}

#[inline]
pub const fn line_separator<'s>() -> &'s str {
    match cfg!(target_os = "windows") {
        true => "\r\n",
        false => "\n",
    }
}

#[inline]
pub const fn endianess<'s>() -> &'s str {
    match cfg!(target_endian = "big") {
        true => "big",
        false => "little",
    }
}

pub fn temp_dir<'s>() -> &'s str {
    todo!()
}
