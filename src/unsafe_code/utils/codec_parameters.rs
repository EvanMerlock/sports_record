use std::convert::{AsMut, AsRef};

use ffmpeg_sys::*;

use unsafe_code::{AsRawPtr, UnsafeError, UnsafeErrorKind, CodecContext};

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
}

impl From<&CodecContext> for CodecParameters {
    fn from(context: &CodecContext) -> CodecParameters {
        let pars = CodecParameters::new();
        unsafe {
            avcodec_parameters_from_context(pars.as_mut_ptr(), context.as_ptr());
        }
        pars
    }
}

impl Clone for CodecParameters {
    fn clone(&self) -> Self { 
        let pars = CodecParameters::new();
        unsafe {
            avcodec_parameters_copy(self.as_ptr(), pars.as_mut_ptr());
        }
    }
}

impl Drop for CodecParameters {
    fn drop(&mut self) {
        unsafe {
            avcodec_parameters_free(self.0)
        }
    }
}