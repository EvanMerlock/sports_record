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
    mjpeg_address: net::SocketAddr,

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

    pub fn get_mjpeg_address(&self) -> &net::SocketAddr {
        &self.mjpeg_address
    }

    pub fn get_camera_settings(&self) -> &CameraConfiguration {
        &self.camera_settings
    }
}

impl Default for ClientConfiguration {
    fn default() -> Self {
        ClientConfiguration {
            name: String::from("CAMERA_NAME"),
            mjpeg_address: net::SocketAddr::new(net::IpAddr::from(net::Ipv4Addr::new(127, 0, 0, 1)), 8080),
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