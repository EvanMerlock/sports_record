use std::marker::{Send, Sync};
use std::convert::{From};
use std::mem;
use std::ops::{Drop, Deref, DerefMut};

use ffmpeg_sys::*;

pub struct Packet(AVPacket);

unsafe impl Send for Packet {}
unsafe impl Sync for Packet {}

impl<'a> From<&'a AVPacket> for Packet {
    fn from(pkt: &AVPacket) -> Packet {
        unsafe {
			let mut new_packet: AVPacket = mem::zeroed();

			av_init_packet(&mut new_packet);
			av_new_packet(&mut new_packet, pkt.size);
            av_copy_packet(&mut new_packet, pkt);

            Packet(new_packet)
        }
    }
}

impl<'a> From<&'a mut AVPacket> for Packet {
    fn from(pkt: &mut AVPacket) -> Packet {
        unsafe {
			let mut new_packet: AVPacket = mem::zeroed();

			av_init_packet(&mut new_packet);
			av_new_packet(&mut new_packet, pkt.size);
            av_copy_packet(&mut new_packet, pkt);

            Packet(new_packet)
        }
    }
}

impl From<Box<AVPacket>> for Packet {
    fn from(pkt: Box<AVPacket>) -> Packet {
        unsafe {
            let ptr = Box::into_raw(pkt);
            let reference = &*ptr;
            Packet::from(reference) 
        }
    }
}

impl From<AVPacket> for Packet {
    fn from(pkt: AVPacket) -> Packet {
        Packet(pkt)
    }
}

impl From<*mut AVPacket> for Packet {
    fn from(pkt: *mut AVPacket) -> Packet {
        unsafe {
            let mut new_packet: AVPacket = mem::zeroed();

		    av_init_packet(&mut new_packet);
		    av_new_packet(&mut new_packet, (*pkt).size);
            av_copy_packet(&mut new_packet, pkt);

            av_packet_unref(pkt);

            Packet(new_packet)
        }
    }
}

impl Drop for Packet {
    fn drop(&mut self) {
        unsafe {
            av_packet_unref(&mut self.0)
        }
    }
}

impl Deref for Packet {
    type Target = AVPacket;

    fn deref(&self) -> &Self::Target {
        &self.0
    } 
}

impl DerefMut for Packet {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    } 
}