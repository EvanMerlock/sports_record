use std::ops::{Deref, DerefMut};
use std::convert::From;

use std::ffi::CString;
use std::slice::from_raw_parts;

use unsafe_code::AsRawPtr;
use unsafe_code::format::{FormatContext, Stream};
use unsafe_code::packet::Packet;

use ffmpeg_sys::*;

pub struct InputContext(FormatContext);

impl InputContext {
    unsafe fn grab_from_input(&mut self) -> *mut AVPacket {
        let pkt = av_packet_alloc();

        av_read_frame(self.as_mut_ptr(), pkt);

        pkt
    }

    pub fn read_input(&mut self) -> Packet {
        unsafe {
            Packet::from(self.grab_from_input())
        }
    }

    unsafe fn get_specific_stream(&self, stream_num: usize) -> Option<Stream> {
        let input_streams = from_raw_parts(self.streams, self.nb_streams as usize);
        if input_streams.len() < stream_num {
            None
        } else {
            Some(Stream::from(input_streams[stream_num]))
        }
}

    pub fn find_input_stream(&self, stream_num: usize) -> Option<Stream> {
        unsafe {
            match self.get_specific_stream(stream_num) {
                Some(item) => Some(item),
                None => None,
            }
        }
    }

    unsafe fn allocate_input_format(format_name: CString) -> *mut AVInputFormat {
        av_find_input_format(format_name.as_ptr())
    }

    pub fn create_input_format<'a>(format_name: CString) -> &'a mut AVInputFormat {
        unsafe {
            &mut *InputContext::allocate_input_format(format_name)
        }
    }
}

impl AsRawPtr<AVFormatContext> for InputContext {
    fn as_ptr(&self) -> *const AVFormatContext {
        self.0.as_ptr()
    }

    fn as_mut_ptr(&mut self) -> *mut AVFormatContext {
        self.0.as_mut_ptr()
    }
}

impl From<FormatContext> for InputContext {
    fn from(ctx: FormatContext) -> InputContext {
        InputContext(ctx)
    }
}

impl AsRef<AVFormatContext> for InputContext {
    fn as_ref(&self) -> &AVFormatContext {
        self.0.as_ref()
    }
}

impl AsMut<AVFormatContext> for InputContext {
    fn as_mut(&mut self) -> &mut AVFormatContext {
        self.0.as_mut()
    }
}

impl Deref for InputContext {
    type Target = AVFormatContext;

    fn deref(&self) -> &Self::Target {
        self.0.as_ref()
    }
}

impl DerefMut for InputContext {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0.as_mut()
    }
}

impl Drop for InputContext {
    fn drop(&mut self) {
        unsafe {
            if !self.as_mut_ptr().is_null() {
                avformat_close_input(&mut self.as_mut_ptr())
            }
        }
    }
}