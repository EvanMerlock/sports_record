use std::marker::{Send, Sync};
use std::convert::{From};
use std::mem;
use std::ops::{Drop, Deref, DerefMut};
use std::io::Write;

use std::slice::{from_raw_parts_mut, from_raw_parts};
use std::ptr;

use unsafe_code::{Rational, AsRawPtr};
use unsafe_code::packet::DataPacket;

use ffmpeg_sys::*;

pub struct Frame(*mut AVFrame);

unsafe impl Send for Frame {}
unsafe impl Sync for Frame {}

impl Frame {
    pub fn null() -> Frame {
        Frame(ptr::null_mut())
    }

    pub fn new() -> Frame {
        unsafe {
            Frame(av_frame_alloc())
        }
    }
}

impl AsRawPtr<AVFrame> for Frame {
    fn as_ptr(&self) -> *const AVFrame {
        self.0 as *const _
    }

    fn as_mut_ptr(&mut self) -> *mut AVFrame {
        self.0
    }
}

impl From<*mut AVFrame> for Frame {
    fn from(frame: *mut AVFrame) -> Frame {
        Frame(frame)
    }
}

impl AsRef<AVFrame> for Frame {
    fn as_ref(&self) -> &AVFrame {
        unsafe {
            &*self.0
        }
    }
}

impl AsMut<AVFrame> for Frame {
    fn as_mut(&mut self) -> &mut AVFrame {
        unsafe {
            &mut *self.0
        }
    }
}

impl Deref for Frame {
    type Target = AVFrame;

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl DerefMut for Frame {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut()
    }
}