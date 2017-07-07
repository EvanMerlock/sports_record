use std::marker::{Send};
use std::convert::{From, AsRef, AsMut};
use std::ops::{Deref, DerefMut};

use unsafe_code::{AsRawPtr};
use unsafe_code::codec::{CodecContext, Codec};

use ffmpeg_sys::*;

pub struct DecodingCodec(Codec);

unsafe impl Send for DecodingCodec {}

impl AsRef<AVCodec> for DecodingCodec {
    fn as_ref(&self) -> &AVCodec {
        self.0.as_ref()
    }
}

impl AsMut<AVCodec> for DecodingCodec {
    fn as_mut(&mut self) -> &mut AVCodec {
        self.0.as_mut()
    }
}

impl From<Codec> for DecodingCodec {
    fn from(codec: Codec) -> DecodingCodec {
        unsafe {
            DecodingCodec(codec)
        }
    }
}

impl AsRawPtr<AVCodec> for DecodingCodec {
    fn as_ptr(&self) -> *const AVCodec {
        self.0.as_ptr()
    }

    fn as_mut_ptr(&mut self) -> *mut AVCodec {
        self.0.as_mut_ptr()
    }
}

pub struct DecodingCodecContext(CodecContext, DecodingCodec);

impl DecodingCodecContext {
    pub fn new(codec: DecodingCodec, context: CodecContext) -> DecodingCodecContext {
        DecodingCodecContext(context, codec)
    }
}

impl AsRef<AVCodecContext> for DecodingCodecContext {
    fn as_ref(&self) -> &AVCodecContext {
        self.0.as_ref()
    }
}

impl AsMut<AVCodecContext> for DecodingCodecContext {
    fn as_mut(&mut self) -> &mut AVCodecContext {
        self.0.as_mut()
    }
}

impl AsRawPtr<AVCodecContext> for DecodingCodecContext {
    fn as_ptr(&self) -> *const AVCodecContext {
        self.0.as_ptr()
    }

    fn as_mut_ptr(&mut self) -> *mut AVCodecContext {
        self.0.as_mut_ptr()
    }
}