use unsafe_code::{CodecContext, AsRawPtr};
use unsafe_code::format::Stream;

use ffmpeg_sys::*;

pub fn put_raw_codecpars_into_stream(stream: &mut Stream, context: *const AVCodecContext) {
    unsafe {
        let ret = avcodec_parameters_from_context((**stream).codecpar, context);
        if ret != 0 {
            println!("raw codecpars ret = {}", ret);
        }
    }
}