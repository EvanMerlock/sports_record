use std::convert::From;
use std::ops::{Deref, DerefMut};

use std::ptr;

use unsafe_code::format::OutputContext;
use unsafe_code::{CodecContext, AsRawPtr};

use ffmpeg_sys::*;

#[derive(Debug)]
pub struct Stream(*mut AVStream);

impl Stream {
    pub fn new<T: AsRef<CodecContext> + Sized>(fmt: &mut OutputContext, code: T) -> Stream {
        unsafe {
            let stream = avformat_new_stream(fmt.as_mut_ptr(), ptr::null());
            let mut s = Stream(stream);
            let _ = s.load_context_into_stream(code.as_ref());
            s
        }
    }

    pub fn load_context_into_stream(&mut self, context: &CodecContext) -> Result<(), i32> {
        unsafe {
            let ret = avcodec_parameters_from_context((**self).codecpar, context.as_ptr());
            if ret != 0 {
                println!("failed to put codec parms into stream: {}", ret);
                return Err(ret);
            } else {
                Ok(())
            }
        }
    }
}

impl AsRawPtr<AVStream> for Stream {
    fn as_ptr(&self) -> *const AVStream {
        self.0 as *const _
    }

    fn as_mut_ptr(&mut self) -> *mut AVStream {
        self.0
    }
}

impl Deref for Stream {
    type Target = AVStream;

    fn deref(&self) -> &Self::Target {
        unsafe {
            &*self.0
        }
    }
}

impl DerefMut for Stream {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe {
            &mut *self.0
        }
    }
}

impl From<*mut AVStream> for Stream {
    fn from(stream: *mut AVStream) -> Stream {
        Stream(stream)
    }
}