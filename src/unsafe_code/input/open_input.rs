use std::ffi::CString;
use std::slice::from_raw_parts;

use unsafe_code::format::InputContext;
use unsafe_code::output::Stream;

use ffmpeg_sys::*;

unsafe fn allocate_input_format(format_name: CString) -> *const AVInputFormat {
    av_find_input_format(format_name.as_ptr())
}

pub fn create_input_format<'a>(format_name: CString) -> &'a AVInputFormat {
    unsafe {
        &*allocate_input_format(format_name)
    }
}

unsafe fn get_specific_stream(format: &InputContext, stream_num: usize) -> Option<Stream> {
    let input_streams = from_raw_parts((*format).streams, (*format).nb_streams as usize);
    if input_streams.len() < stream_num {
        None
    } else {
        Some(Stream::from(input_streams[stream_num]))
    }
}

pub fn find_input_stream(format: &InputContext, stream_num: usize) -> Option<Stream> {
    unsafe {
        match get_specific_stream(format, stream_num) {
            Some(item) => Some(item),
            None => None,
        }
    }
}