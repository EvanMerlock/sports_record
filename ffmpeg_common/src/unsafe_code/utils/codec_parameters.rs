use std::convert::{AsMut, AsRef};

use ffmpeg_sys::*;

use unsafe_code::{AsRawPtr, CodecContext};

pub struct CodecParameters(*mut AVCodecParameters);

impl AsRawPtr<AVCodecParameters> for CodecParameters {
    fn as_mut_ptr(&mut self) -> *mut AVCodecParameters {
        self.0
    }

    fn as_ptr(&self) -> *const AVCodecParameters {
        self.0 as *const _
    }
}

impl AsRef<AVCodecParameters> for CodecParameters {
    fn as_ref(&self) -> &AVCodecParameters {
        unsafe { &*self.0 }
    }
}

impl AsMut<AVCodecParameters> for CodecParameters {
    fn as_mut(&mut self) -> &mut AVCodecParameters {
        unsafe { &mut *self.0 }
    }
}

impl CodecParameters {
    fn new() -> CodecParameters {
        let codecpars_ptr = unsafe {
            avcodec_parameters_alloc()
        };

        CodecParameters(codecpars_ptr)
    }

    pub fn insert_into_context(&mut self, context: &mut CodecContext) -> i32 {
        unsafe {
            avcodec_parameters_to_context(context.as_mut_ptr(), self.as_mut_ptr())
        }
    }
}

impl<'a> From<&'a CodecContext> for CodecParameters {
    fn from(context: &'a CodecContext) -> CodecParameters {
        let mut pars = CodecParameters::new();
        unsafe {
            let _ = avcodec_parameters_from_context(pars.as_mut_ptr(), context.as_ptr());
        }
        pars
    }
}

impl From<*mut AVCodecParameters> for CodecParameters {
    fn from(context: *mut AVCodecParameters) -> CodecParameters {
        CodecParameters(context)
    }
}

impl Clone for CodecParameters {
    fn clone(&self) -> Self { 
        let mut pars = CodecParameters::new();
        unsafe {
            avcodec_parameters_copy(self.0, pars.as_mut_ptr());
        }
        pars
    }
}