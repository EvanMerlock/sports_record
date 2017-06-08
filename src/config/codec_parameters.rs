use std::ops::{Deref, DerefMut, Drop};
use std::convert::{From};
use config::stream_config::StreamConfiguration;
use unsafe_code::CodecContext;
use unsafe_code::output::Stream;

use ffmpeg_sys::*;

pub fn put_raw_codecpars_into_stream(stream: &mut Stream, context: CodecContext) {
    unsafe {
        let ret = avcodec_parameters_from_context((**stream).codecpar, context.as_ptr());
        if ret != 0 {
            println!("raw codecpars ret = {}", ret);
        }
    }
}