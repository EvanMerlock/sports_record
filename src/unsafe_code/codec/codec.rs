use std::marker::{Send};
use std::convert::{From};
use std::ops::{Deref, DerefMut};

use unsafe_code::CodecId;

use ffmpeg_sys::*;

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