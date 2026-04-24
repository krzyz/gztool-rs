#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(unnecessary_transmutes)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

#[cfg(test)]
mod tests {
    use std::{ffi::CString, ptr::null_mut};

    use super::*;

    #[test]
    fn gztool_test() {
        unsafe {
            let mut index: *mut access = null_mut();

            let file_name =
                CString::new(format!("{}/test/lorem.gz", env!("CARGO_MANIFEST_DIR"))).unwrap();
            let index_filename =
                CString::new(format!("{}/test/lorem.gzi", env!("CARGO_MANIFEST_DIR"))).unwrap();

            let indx_n_extraction_opts = INDEX_AND_EXTRACTION_OPTIONS_JUST_CREATE_INDEX;
            let offset = 0;
            let line_number_offset = 0;
            let span_between_points = 0;
            let write_index_to_disk = 1;
            let end_on_first_proper_gzip_eof = 1;
            let always_create_a_complete_index = 1;
            let waiting_time = 4;
            let force_action = 0;
            let wait_for_file_creation = 1;
            let extend_index_with_lines = 0;
            let expected_first_byte = 1;
            let gzip_stream_may_be_damaged = 0;
            let lazy_gzip_stream_patching_at_eof = false;
            let range_number_of_bytes = 0;
            let range_number_of_lines = 0;
            let adjust_index_points_to_byte_boundary = true;
            let compression_factor = 0;
            let continue_tailing_on_eof = false;

            let x = action_create_index(
                file_name.as_c_str().as_ptr().cast_mut(),
                &raw mut index,
                index_filename.as_c_str().as_ptr().cast_mut(),
                indx_n_extraction_opts,
                offset,
                line_number_offset,
                span_between_points,
                write_index_to_disk,
                end_on_first_proper_gzip_eof,
                always_create_a_complete_index,
                waiting_time,
                force_action,
                wait_for_file_creation,
                extend_index_with_lines,
                expected_first_byte,
                gzip_stream_may_be_damaged,
                lazy_gzip_stream_patching_at_eof,
                range_number_of_bytes,
                range_number_of_lines,
                adjust_index_points_to_byte_boundary,
                compression_factor,
                continue_tailing_on_eof,
            );

            println!("{:?}", *index);

            assert_eq!(x, 0);
        }
    }
}
