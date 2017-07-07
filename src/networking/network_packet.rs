use std::convert::From;
use std::io::Write;
use std::fmt;

use config::stream_config::StreamConfiguration;

use unsafe_code::DataPacket;
use unsafe_code::UnsafeError;

use serde_json;
use serde_cbor;

#[derive(Debug)]
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
    pub fn write_to(self, writer: &mut Write) -> Result<(), UnsafeError> {
        match self {
            NetworkPacket::PacketStream(item) => {
                for pkt in item {
                    try!(writer.write(&serde_cbor::to_vec(&pkt)?));
                }
                Ok(())
            },
            NetworkPacket::JSONPayload(item) => {
                let raw_json = try!(serde_json::to_vec(&item));
                try!(writer.write(raw_json.as_slice()));
                Ok(())
            },
            NetworkPacket::PayloadEnd => {
                try!(writer.write(b"===ENDTRANSMISSION==="));
                Ok(())
            }
        }
    }
}