use unsafe_code::{Rational, PixelFormat, CodecId, CodecContext};
use std::convert::From;

use ffmpeg_sys::*;

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct StreamConfiguration {
    pub bit_rate: i64,
    pub height: i32,
    pub width: i32,
    pub frame_rate: Rational,
    pub gop_size: i32,
    pub max_b_frames: i32,
    pub pix_fmt: PixelFormat,
    pub codec_id: CodecId,
    pub time_base: Rational,
}

impl<'a> From<&'a AVStream> for StreamConfiguration {
    fn from(stream: &AVStream) -> StreamConfiguration {
        unsafe {
            let stream_codec_context = &*stream.codec;
            StreamConfiguration {
                bit_rate: stream_codec_context.bit_rate,
                height: stream_codec_context.height,
                width: stream_codec_context.width,
                frame_rate: Rational::from(stream.avg_frame_rate),
                gop_size: stream_codec_context.gop_size,
                max_b_frames: stream_codec_context.max_b_frames,
                pix_fmt: PixelFormat::from(stream_codec_context.pix_fmt),
                codec_id: CodecId::from(stream_codec_context.codec_id),
                time_base: Rational::from(stream_codec_context.time_base),
            }
        }
    }
}

impl<'a> From<&'a CodecContext> for StreamConfiguration {
    fn from(item: &'a CodecContext) -> StreamConfiguration {
            StreamConfiguration {
                bit_rate: item.bit_rate,
                height: item.height,
                width: item.width,
                frame_rate: Rational::from(item.framerate),
                gop_size: item.gop_size,
                max_b_frames: item.max_b_frames,
                pix_fmt: PixelFormat::from(item.pix_fmt),
                codec_id: CodecId::from(item.codec_id),
                time_base: Rational::from(item.time_base),
            }
    }
}