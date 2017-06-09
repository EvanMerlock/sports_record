use std::marker::{Send, Sync};
use std::convert::{From};
use std::mem;
use std::ops::{Drop, Deref, DerefMut};
use std::io::Write;

use std::slice::{from_raw_parts_mut, from_raw_parts};

use unsafe_code::Rational;

use ffmpeg_sys::*;

pub struct Packet(AVPacket);

unsafe impl Send for Packet {}
unsafe impl Sync for Packet {}

impl Packet {
    pub fn new(size: usize) -> Packet {
		unsafe {
			let mut pkt: AVPacket = mem::zeroed();

			av_init_packet(&mut pkt);
			av_new_packet(&mut pkt, size as i32);

			Packet(pkt)
		}
	}

    pub fn as_slice(&self) -> &[u8] {
        unsafe {
            from_raw_parts(self.0.data, self.0.size as usize)
        }
    }

    pub fn rescale_to(&mut self, from_ts: Rational, new_ts: Rational) {
        unsafe {
            av_packet_rescale_ts(&mut **self, from_ts.into(), new_ts.into());
        }
    }
}

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

impl From<Vec<u8>> for Packet {
    fn from(pkt: Vec<u8>) -> Packet {
        unsafe {
            let mut packet = Packet::new(pkt.len());
            let mut data = from_raw_parts_mut(packet.0.data, packet.0.size as usize);

            data.write(pkt.as_ref());

            packet
        }
    }
}

impl From<DataPacket> for Packet {
    fn from(pkt: DataPacket) -> Packet {
        unsafe {
            let mut packet = Packet::new(pkt.packet.len());
            let mut data = from_raw_parts_mut(packet.0.data, packet.0.size as usize);

            data.write(pkt.packet.as_ref());
            packet.pts = pkt.pts;
            packet.dts = pkt.dts;

            packet
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

#[derive(Serialize, Deserialize)]
pub struct DataPacket {
    pub packet: Vec<u8>,
    pub pts: i64,
    pub dts: i64,
}

impl From<Packet> for DataPacket {
    fn from(pkt: Packet) -> DataPacket {
        unsafe {
            DataPacket {
                packet: from_raw_parts(pkt.data, pkt.size as usize).to_vec(),
                pts: pkt.pts,
                dts: pkt.dts,
            }
        }
    }
}