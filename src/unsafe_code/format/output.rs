use std::ops::{Deref, DerefMut};
use std::convert::From;
use std::ffi::CString;
use std::ptr;

use unsafe_code::format::{FormatContext, Stream};
use unsafe_code::{UnsafeError, UnsafeErrorKind, CodecContext, AsRawPtr};
use unsafe_code::packet::Packet;

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

impl OutputContext { 
    unsafe fn allocate_avio_video_file(&mut self, filename: CString) -> Result<(), UnsafeError> {
        let mut o_ctx_ptr: *mut AVIOContext = ptr::null_mut();
        let ret = avio_open(&mut o_ctx_ptr, filename.as_ptr(), AVIO_FLAG_WRITE);
        if ret != 0 {
            return Err(UnsafeError::new(UnsafeErrorKind::AVIOError(ret)));
        }
        self.pb = o_ctx_ptr;
        Ok(())
    }

    pub fn open_video_file(&mut self, filename: &str) -> Result<(), UnsafeError> {
        unsafe {
            self.allocate_avio_video_file(CString::new(filename).unwrap())
        }
    }

    unsafe fn write_header(&mut self) -> Result<(), UnsafeError> {
        let ret = avformat_write_header(self.as_mut_ptr(), &mut ptr::null_mut());
        if ret != 0 {
            return Err(UnsafeError::new(UnsafeErrorKind::WriteHeaderError(ret)));
        }
        Ok(())
    }

    pub fn write_video_header(&mut self) -> Result<(), UnsafeError> {
        unsafe {
            self.write_header()
        }
    }

    unsafe fn write_trailer(&mut self) -> Result<(), UnsafeError> {
        let ret = av_write_trailer(self.as_mut_ptr());
        if ret != 0 {
            return Err(UnsafeError::new(UnsafeErrorKind::AVIOError(ret)));
        }
        Ok(())
    }

    pub fn write_video_trailer(&mut self) -> Result<(), UnsafeError> {
        unsafe {
            self.write_trailer()
        }
    }

    unsafe fn write_frame(&mut self, stream_index: i32, mut pkt: Packet) -> Result<(), UnsafeError> {
        pkt.stream_index = stream_index;
        let ret = av_interleaved_write_frame(self.as_mut_ptr(), &mut *pkt);
        if ret != 0 {
            return Err(UnsafeError::new(UnsafeErrorKind::WriteVideoFrameError(ret)));
        }
        Ok(())
    }

    pub fn write_video_frame(&mut self, stream_index: i32, pkt: Packet) -> Result<(), UnsafeError> {
        unsafe {
            self.write_frame(stream_index, pkt)
        }
    }

    unsafe fn write_null_frame(&mut self) -> Result<(), UnsafeError> {
        let ret = av_interleaved_write_frame(self.as_mut_ptr(), ptr::null_mut());
        if ret != 0 {
            return Err(UnsafeError::new(UnsafeErrorKind::WriteVideoFrameError(ret)));
        }
        Ok(())
    }

    pub fn write_null_video_frame(&mut self) -> Result<(), UnsafeError> {
        unsafe {
            self.write_null_frame()
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