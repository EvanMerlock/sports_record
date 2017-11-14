use std::ops::{Deref, DerefMut};
use std::convert::From;

use unsafe_code::format::{FormatContext, Stream};
use unsafe_code::{CodecContext, AsRawPtr};

use ffmpeg_sys::*;

pub struct OutputContext(FormatContext);

impl OutputContext {
    unsafe fn add_new_stream<T: AsRef<CodecContext> + Sized>(&mut self, pars: T) -> Stream {
        let mut stream = Stream::new(self, pars);
        stream.id = (self.nb_streams - 1) as i32;
        stream
    }

    pub fn create_stream<T: AsRef<CodecContext> + Sized>(&mut self, pars: T) -> Stream {
        unsafe {
            self.add_new_stream(pars)
        }
    }
}

impl AsRawPtr<AVFormatContext> for OutputContext {
    fn as_ptr(&self) -> *const AVFormatContext {
        self.0.as_ptr()
    }

    fn as_mut_ptr(&mut self) -> *mut AVFormatContext {
        self.0.as_mut_ptr()
    }
}

impl From<FormatContext> for OutputContext {

    fn from(ctx: FormatContext) -> OutputContext {
        OutputContext(ctx)
    }

}

impl AsRef<AVFormatContext> for OutputContext {
    fn as_ref(&self) -> &AVFormatContext {
        self.0.as_ref()
    }
}

impl AsMut<AVFormatContext> for OutputContext {
    fn as_mut(&mut self) -> &mut AVFormatContext {
        self.0.as_mut()
    }
}

impl Deref for OutputContext {
    type Target = AVFormatContext;

    fn deref(&self) -> &Self::Target {
        self.0.as_ref()
    }
}

impl DerefMut for OutputContext {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0.as_mut()
    }
}

impl Drop for OutputContext {
    fn drop(&mut self) {
        unsafe {
            if !self.as_mut_ptr().is_null() {
                avformat_free_context(self.as_mut_ptr())
            }
        }
    }
}