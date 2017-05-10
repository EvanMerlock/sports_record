use unsafe_code::input::InputInfo;
use unsafe_code::{UnsafeError, UnsafeErrorKind};

use ffmpeg_sys::*;

use std::ptr;
use std::ffi::CString;
use std::slice::from_raw_parts;

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

unsafe fn allocate_input_information(format: &AVStream) -> InputInfo {
    let time_base = format.time_base;

    let codec = format.codec;
    let width = (*codec).width;
    let height = (*codec).height;
    let frame_rate = (*codec).framerate;

    InputInfo::new(height, width, time_base.num, time_base.den, frame_rate.num, time_base.den)

}

pub fn create_input_information(format: &AVStream) -> InputInfo {
    unsafe {
        allocate_input_information(format)
    }
}

unsafe fn get_specific_stream(format: &AVFormatContext, stream_num: usize) -> Option<*mut AVStream> {
    let input_streams = from_raw_parts((*format).streams, (*format).nb_streams as usize);
    if input_streams.len() < stream_num {
        None
    } else {
        Some(input_streams[stream_num])
    }
}

pub fn find_input_stream<'a>(format: &AVFormatContext, stream_num: usize) -> Option<&'a mut AVStream> {
    unsafe {
        match get_specific_stream(format, stream_num) {
            Some(item) => Some(&mut *item),
            None => None,
        }
    }
}