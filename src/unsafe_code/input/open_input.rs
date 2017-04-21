use unsafe_code::{UnsafeError, UnsafeErrorKind};

use ffmpeg_sys::*;

use std::ptr;
use std::ffi::CString;

unsafe fn allocate_format_context(input_format: &AVInputFormat, input_location: CString) -> Result<*mut AVFormatContext, UnsafeError> {
    let mut input_context_ptr = ptr::null_mut();
    let mut opts_ptr = ptr::null_mut();

    let ret = avformat_open_input(&mut input_context_ptr, input_location.as_ptr(), input_format, &mut opts_ptr); 
    if ret < 0 {
        return Err(UnsafeError::new(UnsafeErrorKind::OpenInput(ret)));
    }

    av_dump_format(input_context_ptr, 0, input_location.as_ptr(), 0);

    Ok(input_context_ptr)
}

pub fn create_format_context<'a>(input_format: &AVInputFormat, input_location: CString) -> Result<&'a mut AVFormatContext, UnsafeError> {
    unsafe {
        match allocate_format_context(input_format, input_location) {
            Ok(fmt) => Ok(&mut *fmt),
            Err(e) => Err(e),
        }
    }
}

unsafe fn allocate_input_format(format_name: CString) -> *const AVInputFormat {
    av_find_input_format(format_name.as_ptr())
}

pub fn create_input_format<'a>(format_name: CString) -> &'a AVInputFormat {
    unsafe {
        &*allocate_input_format(format_name)
    }
}