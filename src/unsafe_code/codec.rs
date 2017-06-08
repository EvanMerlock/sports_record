use std::marker::{Send};
use std::convert::{From};
use std::ops::{Drop, Deref, DerefMut};
use std::ptr;

use unsafe_code::CodecId;

use ffmpeg_sys::*;


pub struct CodecContext(*mut AVCodecContext);

unsafe impl Send for CodecContext {}

impl CodecContext {
    fn new() -> CodecContext {
        unsafe {
            CodecContext(avcodec_alloc_context3(ptr::null()))
        }
    }
}

impl CodecContext {
    pub unsafe fn as_ptr(&self) -> *const AVCodecContext {
        self.0 as *const _
    }

    pub unsafe fn as_mut_ptr(&mut self) -> *mut AVCodecContext {
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

impl Drop for CodecContext {
    fn drop(&mut self) {
        unsafe {
            avcodec_free_context(&mut self.as_mut_ptr());
        }
    }
}


pub struct CodecStorage {
    pub encoding_context: CodecContext,
    pub decoding_context: CodecContext,
}

impl CodecStorage {

    pub fn new(enc: CodecContext, dec: CodecContext) -> CodecStorage {
        CodecStorage {
            encoding_context: enc,
            decoding_context: dec,
        }
    }

}

unsafe impl Send for CodecStorage {}

pub struct Codec(*mut AVCodec);

unsafe impl Send for Codec {}

impl Codec {

    pub fn new_encoder(id: CodecId) -> Codec {
        unsafe {
            Codec(avcodec_find_encoder(*id))
        }
    }

    pub fn new_decoder(id: CodecId) -> Codec {
        unsafe {
            Codec(avcodec_find_decoder(*id))
        }
    }

    pub unsafe fn as_ptr(&self) -> *const AVCodec {
        self.0 as *const _
    }

    pub unsafe fn as_mut_ptr(&mut self) -> *mut AVCodec {
        self.0
    }
}

impl From<*mut AVCodec> for Codec {
    fn from(ctx: *mut AVCodec) -> Codec {
        Codec(ctx)
    }
}

impl Deref for Codec {
    type Target = AVCodec;

    fn deref(&self) -> &Self::Target {
        unsafe {
            &*self.0
        }
    }
}

impl DerefMut for Codec {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe {
            &mut *self.0
        }
    } 
}