use std::ops::{Deref, DerefMut};
use std::convert::From;

use std::ptr;
use std::ffi::CString;

use unsafe_code::format::{OutputContext, InputContext};

use ffmpeg_sys::*;

pub struct FormatContext(*mut AVFormatContext);

unsafe impl Send for FormatContext {}

impl FormatContext {
    pub fn new_output(filename: CString) -> OutputContext {
        unsafe {
            let mut for_ctx_ptr: *mut AVFormatContext = ptr::null_mut();
            avformat_alloc_output_context2(&mut for_ctx_ptr, ptr::null(), ptr::null(), filename.into_raw());
            OutputContext::from(FormatContext(for_ctx_ptr))
        }
    }

    pub fn new_input(input_format: &AVInputFormat, input_location: CString) -> InputContext {
        unsafe {
            let mut input_context_ptr: *mut AVFormatContext = ptr::null_mut();
            let ret = avformat_open_input(&mut input_context_ptr, input_location.as_ptr(), input_format, &mut ptr::null_mut()); 
            if ret < 0 {
                //return Err(UnsafeError::new(UnsafeErrorKind::OpenInput(ret)));
            }

            av_dump_format(input_context_ptr, 0, input_location.as_ptr(), 0);

            InputContext::from(FormatContext(input_context_ptr))
        }
    }
}

impl Deref for FormatContext {
    type Target = AVFormatContext;

    fn deref(&self) -> &Self::Target {
        unsafe {
            &*self.0
        }
    }
}

impl DerefMut for FormatContext {
    fn deref_mut(&mut self) -> &mut Self::Target {
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