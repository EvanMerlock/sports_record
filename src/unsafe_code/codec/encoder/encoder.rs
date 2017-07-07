use std::marker::{Send};
use std::convert::{From, AsRef, AsMut};
use std::ops::{Deref, DerefMut};

use unsafe_code::codec::{CodecContext, Codec};
use unsafe_code::AsRawPtr;

use ffmpeg_sys::*;

pub struct EncodingCodec(Codec);

unsafe impl Send for EncodingCodec {}

impl AsRef<AVCodec> for EncodingCodec {
    fn as_ref(&self) -> &AVCodec {
        self.0.as_ref()
    }
}

impl AsMut<AVCodec> for EncodingCodec {
    fn as_mut(&mut self) -> &mut AVCodec {
        self.0.as_mut()
    }
}

impl From<Codec> for EncodingCodec {
    fn from(codec: Codec) -> EncodingCodec {
        unsafe {
            EncodingCodec(codec)
        }
    }
}

impl AsRawPtr<AVCodec> for EncodingCodec {
    fn as_ptr(&self) -> *const AVCodec {
        self.0.as_ptr()
    }

    fn as_mut_ptr(&mut self) -> *mut AVCodec {
        self.0.as_mut_ptr()
    } 
}

pub struct EncodingCodecContext(CodecContext, EncodingCodec);

impl EncodingCodecContext {
    pub fn new(codec: EncodingCodec, context: CodecContext) -> EncodingCodecContext {
        EncodingCodecContext(context, codec)
    }
}

impl AsRef<AVCodecContext> for EncodingCodecContext {
    fn as_ref(&self) -> &AVCodecContext {
        self.0.as_ref()
    }
}

impl AsMut<AVCodecContext> for EncodingCodecContext {
    fn as_mut(&mut self) -> &mut AVCodecContext {
        self.0.as_mut()
    }
}

impl AsRawPtr<AVCodecContext> for EncodingCodecContext {
    fn as_ptr(&self) -> *const AVCodecContext {
        self.0.as_ptr()
    }

    fn as_mut_ptr(&mut self) -> *mut AVCodecContext {
        self.0.as_mut_ptr()
    }
}

