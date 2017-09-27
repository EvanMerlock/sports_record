use std::convert::From;

use unsafe_code::{Rational, PixelFormat, CodecId, CodecContext, Codec};
use unsafe_code::format::Stream;

use ffmpeg_sys::*;

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub enum CodecVariant {
    Encoding(CodecId),
    Decoding(CodecId),
}

impl<'a> From<&'a Codec> for CodecVariant {
    fn from(codec: &'a Codec) -> CodecVariant {
        match codec.is_encoder() {
            true => CodecVariant::Encoding(CodecId::from(codec.as_ref().id)),
            false => CodecVariant::Decoding(CodecId::from(codec.as_ref().id)),
        }
    }
}

impl <'a> From<&'a AVCodec> for CodecVariant {
    fn from(codec: &'a AVCodec) -> CodecVariant {
        unsafe {
            match av_codec_is_encoder(codec) {
                0 => CodecVariant::Decoding(CodecId::from(codec.id)),
                _ => CodecVariant::Encoding(CodecId::from(codec.id)),
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct StreamConfiguration {
    pub height: i32,
    pub width: i32,
    pub gop_size: i32,
    pub max_b_frames: i32,
    pub pix_fmt: PixelFormat,
    pub codec_id: CodecVariant,
    pub time_base: Rational,
}

impl<'a> From<&'a AVStream> for StreamConfiguration {
    fn from(stream: &AVStream) -> StreamConfiguration {
        unsafe {
            let stream_codec_context = &*stream.codec;
            StreamConfiguration {
                height: stream_codec_context.height,
                width: stream_codec_context.width,
                gop_size: stream_codec_context.gop_size,
                max_b_frames: stream_codec_context.max_b_frames,
                pix_fmt: PixelFormat::from(stream_codec_context.pix_fmt),
                codec_id: CodecVariant::Decoding(CodecId::from(stream_codec_context.codec_id)),
                time_base: Rational::from(stream_codec_context.time_base),
            }
        }
    }
}

impl<'a> From<&'a AVCodecContext> for StreamConfiguration {
    fn from(item: &'a AVCodecContext) -> StreamConfiguration {
        unsafe {
            StreamConfiguration {
                height: item.height,
                width: item.width,
                gop_size: item.gop_size,
                max_b_frames: item.max_b_frames,
                pix_fmt: PixelFormat::from(item.pix_fmt),
                codec_id: CodecVariant::from(&*item.codec),
                time_base: Rational::from(item.time_base),
            }
        }
    }
}

impl<'a> From<&'a CodecContext> for StreamConfiguration {
    fn from(item: &'a CodecContext) -> StreamConfiguration {
        StreamConfiguration::from(item.as_ref())
    }
}

impl <'a> From<&'a Stream> for StreamConfiguration {
    fn from(item: &'a Stream) -> StreamConfiguration {
        StreamConfiguration::from(&**item)
    }
}