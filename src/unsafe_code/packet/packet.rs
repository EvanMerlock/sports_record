use std::marker::{Send, Sync};
use std::convert::{From};
use std::mem;
use std::ptr;
use std::ops::{Drop, Deref, DerefMut};
use std::io::Write;

use std::slice::{from_raw_parts_mut, from_raw_parts};

use unsafe_code::{Rational, AsRawPtr};
use unsafe_code::packet::DataPacket;

use ffmpeg_sys::*;

pub struct Packet(*mut AVPacket);

unsafe impl Send for Packet {}
unsafe impl Sync for Packet {}

impl Packet {
    pub fn new(size: usize) -> Packet {
		unsafe {
			let mut pkt: *mut AVPacket = av_packet_alloc();
			av_new_packet(pkt, size as i32);

			Packet(pkt)
		}
	}

    pub fn as_slice(&self) -> &[u8] {
        unsafe {
            from_raw_parts((*self.0).data, (*self.0).size as usize)
        }
    }

    pub fn rescale_to(&mut self, from_ts: Rational, new_ts: Rational) {
        unsafe {
            av_packet_rescale_ts(&mut **self, from_ts.into(), new_ts.into());
        }
    }
}

impl AsRawPtr<AVPacket> for Packet {
    fn as_ptr(&self) -> *const AVPacket {
        unsafe {
            &*self.0
        }
    }

    fn as_mut_ptr(&mut self) -> *mut AVPacket {
        unsafe {
            &mut *self.0
        }
    }
}

impl<'a> From<&'a AVPacket> for Packet {
    fn from(pkt: &AVPacket) -> Packet {
        unsafe {
			let mut new_packet: Packet = Packet::new(pkt.size as usize);
            av_copy_packet(&mut *new_packet, pkt);
            new_packet
        }
    }
}

impl<'a> From<&'a mut AVPacket> for Packet {
    fn from(pkt: &mut AVPacket) -> Packet {
        unsafe {
			let mut new_packet: Packet = Packet::new(pkt.size as usize);
            av_copy_packet(&mut *new_packet, pkt);
            new_packet
        }
    }
}

impl From<*mut AVPacket> for Packet {
    fn from(pkt: *mut AVPacket) -> Packet {
        Packet(pkt)
    }
}

impl From<Vec<u8>> for Packet {
    fn from(pkt: Vec<u8>) -> Packet {
        unsafe {
            let packet = Packet::new(pkt.len());
            let mut data = from_raw_parts_mut(packet.as_ref().data, packet.as_ref().size as usize);

            let _ = data.write(pkt.as_ref());

            packet
        }
    }
}

impl From<DataPacket> for Packet {
    fn from(pkt: DataPacket) -> Packet {
        unsafe {
            let mut packet = Packet::new(pkt.packet.len());
            let mut data = from_raw_parts_mut(packet.as_ref().data, packet.as_ref().size as usize);

            let _ = data.write(pkt.packet.as_ref());
            packet.pts = pkt.pts;
            packet.dts = pkt.dts;

            packet
        }
    }
}

impl AsRef<AVPacket> for Packet {
    fn as_ref(&self) -> &AVPacket {
        unsafe {
            &*self.0
        }
    }
}

impl AsMut<AVPacket> for Packet {
    fn as_mut(&mut self) -> &mut AVPacket {
        unsafe {
            &mut *self.0
        }
    }
}

impl Clone for Packet {

    fn clone(&self) -> Self {
        let mut pkt = Packet::new(self.as_ref().size as usize);
        pkt.clone_from(self);
        pkt
    }

    fn clone_from(&mut self, source: &Self) {
        unsafe {
            av_copy_packet(self.as_mut_ptr(), &**source);
        }
    }
}

impl Drop for Packet {
    fn drop(&mut self) {
        unsafe {
            av_packet_unref(self.0)
        }
    }
}

impl Deref for Packet {
    type Target = AVPacket;

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    } 
}

impl DerefMut for Packet {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut()
    } 
}