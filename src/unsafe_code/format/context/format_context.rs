use std::ops::{Deref, DerefMut};
use std::convert::From;

use std::ptr;
use std::ffi::CString;

use unsafe_code::{AsRawPtr, UnsafeError, UnsafeErrorKind};
use unsafe_code::format::{OutputContext, InputContext};

use ffmpeg_sys::*;

pub struct FormatContext(*mut AVFormatContext);

unsafe impl Send for FormatContext {}

impl FormatContext {
    // TODO: PROPER ERROR HANDLING
    pub fn new_output(filename: CString) -> OutputContext {
        unsafe {
            let mut for_ctx_ptr: *mut AVFormatContext = ptr::null_mut();
            avformat_alloc_output_context2(&mut for_ctx_ptr, ptr::null(), ptr::null(), filename.into_raw());
            OutputContext::from(FormatContext(for_ctx_ptr))
        }
    }

    pub fn new_input(input_format: &AVInputFormat, input_location: CString) -> Result<InputContext, UnsafeError> {
        unsafe {
            let mut input_context_ptr: *mut AVFormatContext = ptr::null_mut();
            let ret = avformat_open_input(&mut input_context_ptr, input_location.as_ptr(), input_format, &mut ptr::null_mut()); 
            if ret < 0 {
                return Err(UnsafeError::new(UnsafeErrorKind::OpenInput(ret)));
            }

            av_dump_format(input_context_ptr, 0, input_location.as_ptr(), 0);

            Ok(InputContext::from(FormatContext(input_context_ptr)))
        }
    }
}

impl AsRawPtr<AVFormatContext> for FormatContext {
    fn as_ptr(&self) -> *const AVFormatContext {
        self.0 as *const _
    }

    fn as_mut_ptr(&mut self) -> *mut AVFormatContext {
        self.0
    }
}

impl AsRef<AVFormatContext> for FormatContext {
    fn as_ref(&self) -> &AVFormatContext {
        unsafe {
            &*self.0
        }
    }
}

impl AsMut<AVFormatContext> for FormatContext {
    fn as_mut(&mut self) -> &mut AVFormatContext {
        unsafe {
            &mut *self.0
        }
    }
}

impl From<*mut AVFormatContext> for FormatContext {
    fn from(ctx: *mut AVFormatContext) -> FormatContext {
        FormatContext(ctx)
    }
}