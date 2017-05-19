use unsafe_code::{Rational, PixelFormat, CodecId};

use ffmpeg_sys::*;

#[derive(Debug, Serialize, Deserialize)]
pub struct StreamConfiguration {
    pub bit_rate: i64,
    pub height: i32,
    pub width: i32,
    pub frame_rate: Rational,
    pub gop_size: i32,
    pub max_b_frames: i32,
    pub pix_fmt: PixelFormat,
    pub codec_id: CodecId,
}

impl StreamConfiguration {
    pub fn from(stream: &AVStream) -> StreamConfiguration {
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
                codec_id: CodecId::from(stream_codec_context.codec_id)
            }
        }
    }
}