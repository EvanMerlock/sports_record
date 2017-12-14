use std::marker::{Send};
use std::convert::{From};
use std::ops::{Drop, Deref, DerefMut};

use std::ptr;

use unsafe_code::{StreamConfiguration, CodecVariant};
use unsafe_code::{Codec, AsRawPtr, CodecParameters};

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

    pub fn load_parameters_from_codec_parameters<T: AsRawPtr<AVCodecParameters>>(&mut self, params: &T) -> Result<(), i32> {
        unsafe {
            let ret = avcodec_parameters_to_context(self.as_mut_ptr(), params.as_ptr());
            if ret < 0 {
                Err(ret)
            } else {
                Ok(())
            }
        }
    }

    pub fn new_from_stream_configuration(params: &StreamConfiguration) -> CodecContext {
        unsafe {
            let codec: *const AVCodec = match params.codec_id {
                CodecVariant::Encoding(e) => Codec::new_encoder(e).as_ptr(),
                CodecVariant::Decoding(e) => Codec::new_decoder(e).as_ptr(),
            };
            let context_ptr = avcodec_alloc_context3(codec);
            let mut context = CodecContext::from(context_ptr);
            {
                let internal_ref: &mut AVCodecContext = <CodecContext as AsMut<AVCodecContext>>::as_mut(&mut context);
                internal_ref.height = params.height;
                internal_ref.width = params.width;
                internal_ref.time_base = params.time_base.into();
                internal_ref.gop_size = params.gop_size;
                internal_ref.max_b_frames = params.max_b_frames;
                internal_ref.pix_fmt = *params.pix_fmt;
            }
            context
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
        let mut ctx = CodecContext::new_from_stream_configuration(&StreamConfiguration::from(self));
        unsafe {
            let mut codec_pars = CodecParameters::from(self);
            avcodec_parameters_to_context(ctx.as_mut_ptr(), codec_pars.as_mut_ptr());
        }
        ctx
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
            if !self.as_mut_ptr().is_null() {
                avcodec_free_context(&mut self.as_mut_ptr());
            }
        }
    }
}