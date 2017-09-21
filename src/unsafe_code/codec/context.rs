use std::marker::{Send};
use std::convert::{From};
use std::ops::{Drop, Deref, DerefMut};

use std::ptr;

use unsafe_code::{UnsafeError, UnsafeErrorKind, Codec, AsRawPtr};
use unsafe_code::codec::{EncodingCodec, DecodingCodec, EncodingCodecContext, DecodingCodecContext};

use ffmpeg_sys::*;

pub struct CodecContext(*mut AVCodecContext);

unsafe impl Send for CodecContext {}

impl CodecContext {
    pub fn new() -> CodecContext {
        unsafe {
            CodecContext(avcodec_alloc_context3(ptr::null()))
        }
    }

    pub fn new_codec_based_context<T: AsRawPtr<AVCodec> + Sized>(codec: &T) -> CodecContext {
        unsafe {
            CodecContext(avcodec_alloc_context3(codec.as_ptr()))
        }
    }
}

impl AsRawPtr<AVCodecContext> for CodecContext {
    fn as_ptr(&self) -> *const AVCodecContext {
        self.0 as *const _
    }

    fn as_mut_ptr(&mut self) -> *mut AVCodecContext {
        self.0
    }
}

impl From<*mut AVCodecContext> for CodecContext {
    fn from(ctx: *mut AVCodecContext) -> CodecContext {
        CodecContext(ctx)
    }
}

impl Clone for CodecContext {
    fn clone(&self) -> Self {
        let mut ctx = CodecContext::new();
        ctx.clone_from(self);

        ctx
    }

    fn clone_from(&mut self, source: &Self) {
        unsafe {
            avcodec_copy_context(self.as_mut_ptr(), source.as_ptr());
        }
    }
}

impl Deref for CodecContext {
    type Target = AVCodecContext;

    fn deref(&self) -> &Self::Target {
        unsafe {
            &*self.0
        }
    }
}

impl DerefMut for CodecContext {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe {
            &mut *self.0
        }
    }
}

impl AsRef<AVCodecContext> for CodecContext {
    fn as_ref(&self) -> &AVCodecContext {
        unsafe {
            &*self.0
        }
    }
}

impl AsMut<AVCodecContext> for CodecContext {
    fn as_mut(&mut self) -> &mut AVCodecContext {
        unsafe {
            &mut *self.0
        }
    }
}

impl Drop for CodecContext {
    fn drop(&mut self) {
        unsafe {
            println!("dropping codec context");
            avcodec_free_context(&mut self.as_mut_ptr());
        }
    }
}