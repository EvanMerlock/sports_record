use std::ops::{Deref, DerefMut};
use std::convert::From;

use unsafe_code::format::FormatContext;
use unsafe_code::packet::Packet;

use ffmpeg_sys::*;

pub struct InputContext(FormatContext);

impl InputContext {
    unsafe fn grab_from_input(&mut self) -> *mut AVPacket {
        let pkt = av_packet_alloc();

        av_read_frame(&mut **self, pkt);

        pkt
    }

    pub fn read_input(&mut self) -> Packet {
        unsafe {
            Packet::from(self.grab_from_input())
        }
    }
}

impl From<FormatContext> for InputContext {
    fn from(ctx: FormatContext) -> InputContext {
        InputContext(ctx)
    }
}

impl Deref for InputContext {
    type Target = AVFormatContext;

    fn deref(&self) -> &Self::Target {
        &*self.0
    }
}

impl DerefMut for InputContext {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut *self.0
    }
}