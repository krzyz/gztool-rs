use std::{ffi::CString, marker::PhantomData, path::Path, ptr::null_mut};

use libc::{FILE, fclose, fmemopen, free, open_memstream};

use crate::{Access, DEFAULT_SPAN, GzToolError, GzToolResult};

pub(crate) struct WriteBufferFile {
    pub(crate) file: *mut FILE,
    pub(crate) buffer: Box<*mut i8>,
    pub(crate) len: Box<usize>,
}

impl WriteBufferFile {
    fn new() -> Self {
        // note: this box stores a pointer to a buffer that will be
        // set/allocated by open_memstream so that buffer needs to be
        // freed manually in the Drop implementation for this struct
        let mut buffer = Box::new(null_mut());
        let mut len = Box::new(0);
        let file = unsafe { open_memstream(&raw mut *buffer, &raw mut *len) };

        Self { file, buffer, len }
    }

    fn into_vec(self) -> Vec<u8> {
        let slice = unsafe { std::slice::from_raw_parts(*self.buffer as *const u8, *self.len) };

        // TODO: Expose as a slice directly instead of copying to vector
        slice.to_vec()
    }
}

impl Drop for WriteBufferFile {
    fn drop(&mut self) {
        unsafe {
            fclose(self.file);
            free(self.buffer.cast());
        }
    }
}

pub struct BufferAsFile<'a> {
    file: *mut FILE,
    _lifetime: PhantomData<&'a ()>,
}

impl<'a> BufferAsFile<'a> {
    fn new(buffer: &'a [u8]) -> GzToolResult<Self> {
        let in_mode = CString::new("rb")?;
        let file = unsafe { fmemopen(buffer.as_ptr() as _, buffer.len(), in_mode.as_ptr()) };

        Ok(Self {
            file,
            _lifetime: PhantomData::<&'a ()>,
        })
    }
}

impl Drop for BufferAsFile<'_> {
    fn drop(&mut self) {
        unsafe { fclose(self.file) };
    }
}

#[derive(Debug)]
pub(crate) enum BuildIndexOption {
    CreateIndex,
    Decompress {
        starting_decompressed_byte: u64,
        decompressed_size: Option<u64>,
        starting_compressed_byte: Option<u64>,
        access: Access,
    },
}

pub(crate) fn decompress_or_build_index(
    compressed_buf: &[u8],
    index_path: &Path,
    span: Option<u64>,
    option: BuildIndexOption,
) -> GzToolResult<Vec<u8>> {
    let write_buffer_file = WriteBufferFile::new();

    let empty_string = CString::new("")?;
    let index_path = CString::new(
        index_path
            .to_str()
            .ok_or_else(|| GzToolError::PathToStringError(index_path.into()))?,
    )?;
    let (
        offset,
        range_number_of_bytes,
        indx_n_extraction_opts,
        write_index_to_disk,
        access,
        expected_first_byte,
    ) = match option {
        BuildIndexOption::CreateIndex => (
            0,
            0,
            crate::ffi::INDEX_AND_EXTRACTION_OPTIONS_JUST_CREATE_INDEX,
            1,
            None,
            1,
        ),
        BuildIndexOption::Decompress {
            starting_decompressed_byte,
            decompressed_size,
            starting_compressed_byte,
            access,
        } => (
            starting_decompressed_byte,
            decompressed_size.unwrap_or(0),
            crate::ffi::INDEX_AND_EXTRACTION_OPTIONS_EXTRACT_FROM_BYTE,
            0,
            Some(access),
            starting_compressed_byte.unwrap_or(1),
        ),
    };
    let line_number_offset = 0;
    let span = span.unwrap_or(DEFAULT_SPAN);
    let end_on_first_proper_gzip_eof = 0;
    let always_create_a_complete_index = 0;
    let waiting_time = 4;
    let extend_index_with_lines = 0;
    let gzip_stream_may_be_damaged = 0;
    let lazy_gzip_stream_patching_at_eof = false;
    let range_number_of_lines = 0;
    let adjust_index_points_to_byte_boundary = false;
    let continue_tailing_on_eof = false;

    let mut access_ptr = if let Some(mut access) = access {
        std::mem::take(&mut access.inner)
    } else {
        null_mut()
    };

    let in_as_file = BufferAsFile::new(compressed_buf)?;

    unsafe {
        let res = crate::ffi::decompress_and_build_index(
            in_as_file.file.cast::<crate::ffi::_IO_FILE>(),
            write_buffer_file.file.cast::<crate::ffi::_IO_FILE>(),
            empty_string.as_ptr().cast_mut(),
            span,
            &raw mut access_ptr,
            indx_n_extraction_opts,
            offset,
            line_number_offset,
            index_path.as_ptr().cast_mut(),
            write_index_to_disk,
            end_on_first_proper_gzip_eof,
            always_create_a_complete_index,
            waiting_time,
            extend_index_with_lines,
            expected_first_byte,
            gzip_stream_may_be_damaged,
            lazy_gzip_stream_patching_at_eof,
            range_number_of_bytes,
            range_number_of_lines,
            adjust_index_points_to_byte_boundary,
            continue_tailing_on_eof,
        );

        // Wrap returned (possibly null) access so it's cleared properly
        let _ = Access::new(access_ptr);

        if res.error >= 0 {
            Ok(write_buffer_file.into_vec())
        } else {
            Err(GzToolError::ZlibError(res.error.into()))
        }
    }
}
