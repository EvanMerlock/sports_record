
use std::slice::from_raw_parts;

use unsafe_code::packet::Packet;

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