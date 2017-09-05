use std::convert::From;
use std::io::Write;
use std::fmt;

use config::stream_config::StreamConfiguration;

use unsafe_code::DataPacket;
use unsafe_code::UnsafeError;

use serde::Serialize;
use serde_json;

#[derive(Debug, Serialize, Deserialize)]
pub enum NetworkPacket {
    PacketStream(Vec<DataPacket>),
    JSONPayload(StreamConfiguration),
    PayloadEnd,
}

impl From<Vec<DataPacket>> for NetworkPacket {
    fn from(item: Vec<DataPacket>) -> NetworkPacket {
        NetworkPacket::PacketStream(item)
    }
}

impl From<StreamConfiguration> for NetworkPacket {
    fn from(item: StreamConfiguration) -> NetworkPacket {
        NetworkPacket::JSONPayload(item)
    }
}

impl NetworkPacket {
    pub fn write_to(&self, writer: &mut Write) -> Result<(), UnsafeError> {
        let mut buf = Vec::new();
        self.serialize(&mut serde_json::Serializer::new(&mut buf))?;
        writer.write(&buf);
        Ok(())
    }
}