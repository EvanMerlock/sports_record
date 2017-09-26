use std::marker::{Send};
use std::convert::{From};
use std::ops::{Deref, DerefMut};

use unsafe_code::{CodecId, AsRawPtr};
use unsafe_code::codec::{EncodingCodec, DecodingCodec};

use ffmpeg_sys::*;

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

    pub fn is_encoder(&self) -> bool {
		unsafe {
			av_codec_is_encoder(self.as_ptr()) != 0
		}
	}

	pub fn is_decoder(&self) -> bool {
		unsafe {
			av_codec_is_decoder(self.as_ptr()) != 0
		}
    }

    pub fn get_codec_id(&self) -> CodecId {
        CodecId::from(self.as_ref().id)
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

impl AsRawPtr<AVCodec> for Codec {
    fn as_ptr(&self) -> *const AVCodec {
        self.0 as *const _
    }

    fn as_mut_ptr(&mut self) -> *mut AVCodec {
        self.0
    }
}

impl Clone for Codec {
    fn clone(&self) -> Self {
        if self.is_encoder() {
            Codec::from(Codec::new_encoder(self.get_codec_id()).as_mut_ptr())
        } else {
            Codec::from(Codec::new_decoder(self.get_codec_id()).as_mut_ptr())
        }
    }
}