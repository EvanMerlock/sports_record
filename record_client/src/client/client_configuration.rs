use std::net;
use std::io::{Write, Read};
use std::io;
use std::fs::File;
use toml;
use std::fmt;
use std::error::Error;
use std::ffi::CString;
use std::default::Default;

#[derive(Debug)]
pub enum ClientConfigurationError {
    TOMLDEError(toml::de::Error),
    TOMLSERError(toml::ser::Error),
    IOError(io::Error),
}

impl fmt::Display for ClientConfigurationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ClientConfigurationError::TOMLDEError(ref e) => e.fmt(f),
            ClientConfigurationError::TOMLSERError(ref e) => e.fmt(f),
            ClientConfigurationError::IOError(ref e) => e.fmt(f),
        }
    } 
}

impl Error for ClientConfigurationError {
    fn description(&self) -> &str {
        "ClientConfigurationError"
    }

    fn cause(&self) -> Option<&Error> {
        None
    }
}

impl From<io::Error> for ClientConfigurationError {
    fn from(e: io::Error) -> ClientConfigurationError {
        ClientConfigurationError::IOError(e)
    }
}

impl From<toml::de::Error> for ClientConfigurationError {
    fn from(e: toml::de::Error) -> ClientConfigurationError {
        ClientConfigurationError::TOMLDEError(e)
    }
}

impl From<toml::ser::Error> for ClientConfigurationError {
    fn from(e: toml::ser::Error) -> ClientConfigurationError {
        ClientConfigurationError::TOMLSERError(e)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ClientConfiguration {
    name: String,

    ip_settings: IpConfiguration,
    camera_settings: CameraConfiguration,
}

impl ClientConfiguration {
    pub fn from(mut file: File) -> Result<ClientConfiguration, ClientConfigurationError> {
        let mut file_contents = Vec::new();
        file.read_to_end(&mut file_contents)?;
        Ok(toml::from_slice(&file_contents)?)
    }

    pub fn write_to(&self, mut file: File) -> Result<(), ClientConfigurationError> {
        file.write(&toml::to_vec(self)?)?;
        Ok(())
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_ip_settings(&self) -> &IpConfiguration {
        &self.ip_settings
    }

    pub fn get_camera_settings(&self) -> &CameraConfiguration {
        &self.camera_settings
    }
}

impl Default for ClientConfiguration {
    fn default() -> Self {
        ClientConfiguration {
            name: String::from("CAMERA_NAME"),
            ip_settings: IpConfiguration::default()
            camera_settings: CameraConfiguration::default(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CameraConfiguration {
    input_type: String,
    location: String,
}

impl CameraConfiguration {
    pub fn get_input_type(&self) -> CString {
        CString::new(self.input_type.as_bytes()).expect("Failed to create CString")
    }

    pub fn get_camera_location(&self) -> CString {
        CString::new(self.location.as_bytes()).expect("Failed to create CString")
    }
}

impl Default for CameraConfiguration {
    fn default() -> Self {
        CameraConfiguration {
            input_type: String::from("v4l2"),
            location: String::from("/dev/video0")
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IpConfiguration {
    websocket_bind_address: SocketAddr,
    http_bind_address: SocketAddr,
    discovery_port: u16,
    multicast_address: net::Ipv4Addr,
}

impl IpConfiguration {
    pub fn get_multicast_ip(&self) -> net::Ipv4Addr {
        self.multicast_address.clone()
    }

    pub fn get_ws_bind_address(&self) -> SocketAddr {
        self.websocket_bind_address.clone()
    }

    pub fn get_http_bind_address(&self) -> SocketAddr {
        self.http_bind_address.clone()
    }

    pub fn get_discovery_port(&self) -> u16 {
        self.discovery_port
    }
}

impl Default for IpConfiguration {
    fn default() -> Self {
        IpConfiguration {
            multicast_address: net::Ipv4Addr::new(224, 0, 0, 12),
            websocket_bind_address: SocketAddr::from(net::SocketAddrV4::new(net::Ipv4Addr::localhost(), 4000)),
            http_bind_address: SocketAddr::from(net::SocketAddrV4::new(net::Ipv4Addr::localhost(), 8070)),
            discovery_port: 9000,
        }
    }
}