use std::convert::From;
use std::io::Write;

use unsafe_code::DataPacket;
use unsafe_code::UnsafeError;

use networking::NetworkConfiguration;

use serde_json;

#[derive(Debug, Serialize, Deserialize)]
pub enum NetworkPacket {
    PacketStream(Vec<DataPacket>),
    JSONPayload(NetworkConfiguration),
    PayloadEnd,
}

impl From<Vec<DataPacket>> for NetworkPacket {
    fn from(item: Vec<DataPacket>) -> NetworkPacket {
        NetworkPacket::PacketStream(item)
    }
}

impl From<NetworkConfiguration> for NetworkPacket {
    fn from(item: NetworkConfiguration) -> NetworkPacket {
        NetworkPacket::JSONPayload(item)
    }
}

impl NetworkPacket {
    pub fn write_to(&self, writer: &mut Write) -> Result<(), UnsafeError> {
        let vec = serde_json::to_vec(self)?;
        writer.write(&vec)?;
        Ok(())
    }
}