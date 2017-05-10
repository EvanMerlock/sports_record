use ffmpeg_sys::*;

pub struct StreamConfiguration {
    pub bit_rate: i64,
    pub height: i32,
    pub width: i32,
    pub frame_rate: AVRational,
    pub gop_size: i32,
    pub max_b_frames: i32,
    pub pix_fmt: AVPixelFormat,
    pub codec_id: AVCodecID,
}

impl StreamConfiguration {
    pub fn from(stream: &AVStream) -> StreamConfiguration {
        unsafe {
            let stream_codec_context = &*stream.codec;
            StreamConfiguration {
                bit_rate: stream_codec_context.bit_rate,
                height: stream_codec_context.height,
                width: stream_codec_context.width,
                frame_rate: stream.avg_frame_rate,
                gop_size: stream_codec_context.gop_size,
                max_b_frames: stream_codec_context.max_b_frames,
                pix_fmt: stream_codec_context.pix_fmt,
                codec_id: stream_codec_context.codec_id
            }
        }
    }
}