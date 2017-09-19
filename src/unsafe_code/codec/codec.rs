use std::marker::{Send};
use std::convert::{From};
use std::ops::{Deref, DerefMut};

use unsafe_code::{CodecId, AsRawPtr};
use unsafe_code::codec::{EncodingCodec, DecodingCodec};

use ffmpeg_sys::*;

#[derive(Clone)]
pub struct Codec(*mut AVCodec);

unsafe impl Send for Codec {}

impl Codec {

    pub fn new_encoder(id: CodecId) -> EncodingCodec {
        unsafe {
            EncodingCodec::from(Codec(avcodec_find_encoder(*id)))
        }
    }

    pub fn new_decoder(id: CodecId) -> DecodingCodec {
        unsafe {
            DecodingCodec::from(Codec(avcodec_find_decoder(*id)))
        }
    }
}

impl From<*mut AVCodec> for Codec {
    fn from(ctx: *mut AVCodec) -> Codec {
        Codec(ctx)
    }
}

impl AsRef<AVCodec> for Codec {
    fn as_ref(&self) -> &AVCodec {
        unsafe {
            &*self.0
        }
    }
}

impl AsMut<AVCodec> for Codec {
    fn as_mut(&mut self) -> &mut AVCodec {
        unsafe {
            &mut *self.0
        }
    }
}

impl AsRawPtr<AVCodec> for Codec{
    fn as_ptr(&self) -> *const AVCodec {
        self.0 as *const _
    }

    fn as_mut_ptr(&mut self) -> *mut AVCodec {
        self.0
    }
}