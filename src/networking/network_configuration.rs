use std::net::SocketAddr;

use unsafe_code::StreamConfiguration;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NetworkConfiguration {
    pub stream_configuration: StreamConfiguration,
    pub websocket_address: SocketAddr,
}

impl NetworkConfiguration {
    pub fn new(stream_config: StreamConfiguration, ws_addr: SocketAddr) -> NetworkConfiguration {
        NetworkConfiguration {
            stream_configuration: stream_config,
            websocket_address: ws_addr,
        }
    }
}