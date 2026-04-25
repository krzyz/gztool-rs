#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(unnecessary_transmutes)]

mod internal;

pub mod ffi {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

use std::{
    ffi::CString,
    marker::PhantomData,
    path::{Path, PathBuf},
    ptr::NonNull,
};

use libc::{fclose, fopen};
use thiserror::Error;

use crate::{
    ffi::{VERBOSITY_LEVEL, verbosity_level},
    internal::{BuildIndexOption, decompress_or_build_index},
};

const DEFAULT_SPAN: u64 = 10_485_760;

#[derive(Error, Debug)]
pub enum GzToolError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("FFI error: {0}")]
    Ffi(#[from] std::ffi::NulError),
    #[error("Error opening index file: {0}")]
    IndexFileError(PathBuf),
    #[error("Error converting path: {0} to string")]
    PathToStringError(PathBuf),
    #[error("Zlib error code: {0}")]
    ZlibError(ZlibError),
}

#[derive(Error, Debug)]
pub enum ZlibError {
    #[error("Errno")]
    Errno = -1,
    #[error("Stream error")]
    StreamError = -2,
    #[error("Data error")]
    DataError = -3,
    #[error("Memory error")]
    MemError = -4,
    #[error("Buffer error")]
    BufError = -5,
    #[error("VersionError")]
    VersionError = -6,
    #[error("Unknown Error")]
    UnknownError = 0,
}

impl From<i32> for ZlibError {
    fn from(value: i32) -> Self {
        match value {
            -1 => ZlibError::Errno,
            -2 => ZlibError::StreamError,
            -3 => ZlibError::DataError,
            -4 => ZlibError::MemError,
            -5 => ZlibError::BufError,
            -6 => ZlibError::VersionError,
            _ => ZlibError::UnknownError,
        }
    }
}

pub type GzToolResult<T> = std::result::Result<T, GzToolError>;

#[allow(clippy::cast_possible_wrap)]
pub enum VerbosityLevel {
    None = ffi::VERBOSITY_LEVEL_VERBOSITY_NONE as isize,
    Normal = ffi::VERBOSITY_LEVEL_VERBOSITY_NORMAL as isize,
    Excessive = ffi::VERBOSITY_LEVEL_VERBOSITY_EXCESSIVE as isize,
    Maniac = ffi::VERBOSITY_LEVEL_VERBOSITY_MANIAC as isize,
    Crazy = ffi::VERBOSITY_LEVEL_VERBOSITY_CRAZY as isize,
    Nuts = ffi::VERBOSITY_LEVEL_VERBOSITY_NUTS as isize,
}

impl From<VerbosityLevel> for ffi::VERBOSITY_LEVEL {
    fn from(value: VerbosityLevel) -> Self {
        value as u32
    }
}

pub struct Point<'a> {
    inner: NonNull<ffi::point>,
    _lifetime: PhantomData<&'a ()>,
}

impl<'a> Point<'a> {
    fn new(inner: &'a ffi::point) -> Self {
        let ptr = std::ptr::from_ref::<ffi::point>(inner);
        unsafe {
            Self {
                inner: NonNull::new_unchecked(ptr.cast_mut()),
                _lifetime: PhantomData::<&'a ()>,
            }
        }
    }

    pub fn decompressed_index(&self) -> u64 {
        unsafe { self.inner.as_ref().out }
    }

    pub fn compressed_index(&self) -> u64 {
        unsafe { self.inner.as_ref().in_ }
    }

    pub fn bits(&self) -> u32 {
        unsafe { self.inner.as_ref().bits }
    }

    pub fn window_beginning(&self) -> u64 {
        unsafe { self.inner.as_ref().window_beginning }
    }

    pub fn window_size(&self) -> u32 {
        unsafe { self.inner.as_ref().window_size }
    }
}

pub struct PointIter<'a> {
    inner: std::slice::Iter<'a, ffi::point>,
}

impl<'a> Iterator for PointIter<'a> {
    type Item = Point<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(Point::new)
    }
}

#[derive(Debug)]
pub struct Access {
    inner: *mut ffi::access,
}

impl Access {
    fn new(ffi_access: *mut ffi::access) -> Self {
        Self { inner: ffi_access }
    }

    pub fn list_iter(&self) -> Option<PointIter<'_>> {
        if self.inner.is_null() {
            return None;
        }

        let access = unsafe { &*self.inner };
        #[allow(clippy::cast_possible_truncation)]
        let inner = unsafe { std::slice::from_raw_parts(access.list, access.have as usize) };
        Some(PointIter {
            inner: inner.iter(),
        })
    }
}

impl Drop for Access {
    fn drop(&mut self) {
        unsafe {
            ffi::free_index(self.inner);
        }
    }
}

pub fn set_gztool_stdout_verbosity(level: VerbosityLevel) {
    unsafe {
        verbosity_level = level as VERBOSITY_LEVEL;
    }
}

pub fn deserialize_index_from_file(index_path: &Path) -> Option<Access> {
    let load_windows = 1;
    let extend_index_with_lines = 0;

    let index_name = CString::new(index_path.to_str()?).ok()?;
    let mode = CString::new("rb").ok()?;

    let ffi_access = unsafe {
        let index_file = fopen(index_name.as_ptr(), mode.as_ptr());

        let ffi_access = ffi::deserialize_index_from_file(
            index_file.cast::<ffi::_IO_FILE>(),
            load_windows,
            index_name.as_ptr().cast_mut(),
            extend_index_with_lines,
        );

        fclose(index_file);

        ffi_access
    };

    if ffi_access.is_null() {
        None
    } else {
        Some(Access::new(ffi_access))
    }
}

pub fn build_index(
    compressed_buf: &[u8],
    index_path: &Path,
    span: Option<u64>,
) -> GzToolResult<()> {
    decompress_or_build_index(
        compressed_buf,
        index_path,
        span,
        BuildIndexOption::CreateIndex,
    )?;

    Ok(())
}

pub fn decompress(
    compressed_buf: &[u8],
    index_path: &Path,
    span: Option<u64>,
    starting_decompressed_byte: u64,
    decompressed_size: Option<u64>,
    starting_compressed_byte: Option<u64>,
    access: Access,
) -> GzToolResult<Vec<u8>> {
    decompress_or_build_index(
        compressed_buf,
        index_path,
        span,
        BuildIndexOption::Decompress {
            starting_decompressed_byte,
            decompressed_size,
            starting_compressed_byte,
            access,
        },
    )
}

#[cfg(test)]
mod tests {
    use std::{error::Error, io::Write};

    use flate2::{Compression, write::GzEncoder};
    use tempfile::NamedTempFile;

    use super::*;

    #[test]
    fn gztool_test() -> Result<(), Box<dyn Error>> {
        set_gztool_stdout_verbosity(VerbosityLevel::None);

        let tmp_index_file = NamedTempFile::new()?;

        let how_many = 100_000;

        let lorem_ipsum = b"Lorem ipsum dolor sit amet, consectetur adipiscing
elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.";

        let mut uncompressed_source = Vec::with_capacity(how_many * lorem_ipsum.len());
        for i in 0..how_many {
            uncompressed_source.extend_from_slice(lorem_ipsum);
            uncompressed_source.extend(i.to_ne_bytes());
        }

        let mut decoder = GzEncoder::new(Vec::new(), Compression::default());

        decoder.write_all(&uncompressed_source)?;

        let compressed = decoder.finish()?;

        let span = Some(1024 * 1024);
        //let span = None;

        let bytes_start = 5_000_000_usize;
        let bytes_end = 7_000_000_usize;

        build_index(&compressed, tmp_index_file.path(), span)?;

        let access = deserialize_index_from_file(tmp_index_file.path())
            .expect("Error reading index from file");

        let decompressed = decompress(
            &compressed,
            tmp_index_file.path(),
            span,
            bytes_start as u64,
            Some((bytes_end - bytes_start) as u64),
            None,
            access,
        )?;

        assert_eq!(&decompressed, &uncompressed_source[bytes_start..bytes_end]);

        // Read index again as access might be unusable after earlier call
        let access = deserialize_index_from_file(tmp_index_file.path())
            .expect("Error reading index from file");

        // Get which window(s) are needed for decompression
        let (range_start, range_end, decompressed_start, decompressed_end) = {
            let nums = access
                .list_iter()
                .ok_or("Access struct doesn't contain list of point")?
                .map(|p| (p.decompressed_index(), p.compressed_index()))
                .collect::<Vec<_>>();

            let mut compressed_before_start = None;
            let mut compressed_after_end = None;
            let mut decompressed_before_start = None;
            let mut decompressed_after_end = None;
            for &(uncompressed_byte, compressed_byte) in &nums {
                if uncompressed_byte < bytes_start as u64 {
                    compressed_before_start = Some(compressed_byte);
                    decompressed_before_start = Some(uncompressed_byte);
                }

                if uncompressed_byte > bytes_end as u64 {
                    compressed_after_end = Some(compressed_byte);
                    decompressed_after_end = Some(uncompressed_byte);
                    break;
                }
            }

            (
                compressed_before_start,
                compressed_after_end,
                decompressed_before_start,
                decompressed_after_end,
            )
        };

        let range_start = range_start.unwrap();

        let decompressed_size =
            decompressed_start.and_then(|start| decompressed_end.map(|end| end - start));

        let decompressed_start = decompressed_start.unwrap();

        #[allow(clippy::cast_possible_truncation)]
        let decompressed_part = decompress(
            &compressed[(range_start as usize - 1)
                ..(range_end.unwrap_or(compressed.len() as u64) as usize)],
            tmp_index_file.path(),
            span,
            decompressed_start,
            decompressed_size,
            Some(range_start),
            access,
        )?;

        assert_eq!(
            &decompressed_part[(bytes_start - decompressed_start as usize)
                ..(bytes_end - decompressed_start as usize)],
            &uncompressed_source[bytes_start..bytes_end]
        );

        Ok(())
    }
}
