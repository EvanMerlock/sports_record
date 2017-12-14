use std::net;
use std::io::{Write, Read};
use std::io;
use std::fs::File;
use std::path::{PathBuf, Path};
use toml;
use std::fmt;
use std::error::Error;
use std::ffi::CString;
use std::default::Default;
use std::net::SocketAddr;

#[derive(Debug)]
pub enum ServerConfigurationError {
    TOMLDEError(toml::de::Error),
    TOMLSERError(toml::ser::Error),
    IOError(io::Error),
}

impl fmt::Display for ServerConfigurationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ServerConfigurationError::TOMLDEError(ref e) => e.fmt(f),
            ServerConfigurationError::TOMLSERError(ref e) => e.fmt(f),
            ServerConfigurationError::IOError(ref e) => e.fmt(f),
        }
    } 
}

impl Error for ServerConfigurationError {
    fn description(&self) -> &str {
        "ServerConfigurationError"
    }

    fn cause(&self) -> Option<&Error> {
        None
    }
}

impl From<io::Error> for ServerConfigurationError {
    fn from(e: io::Error) -> ServerConfigurationError {
        ServerConfigurationError::IOError(e)
    }
}

impl From<toml::de::Error> for ServerConfigurationError {
    fn from(e: toml::de::Error) -> ServerConfigurationError {
        ServerConfigurationError::TOMLDEError(e)
    }
}

impl From<toml::ser::Error> for ServerConfigurationError {
    fn from(e: toml::ser::Error) -> ServerConfigurationError {
        ServerConfigurationError::TOMLSERError(e)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerConfiguration {
    team_name: String,

    clip_server_port: SocketAddr,
    web_server_port: SocketAddr,
    discovery_port: SocketAddr,

    output_directory: PathBuf,
    database_name: String,
}

impl ServerConfiguration {
    pub fn from(mut file: File) -> Result<ServerConfiguration, ServerConfigurationError> {
        let mut file_contents = Vec::new();
        file.read_to_end(&mut file_contents)?;
        Ok(toml::from_slice(&file_contents)?)
    }

    pub fn write_to(&self, mut file: File) -> Result<(), ServerConfigurationError> {
        file.write(&toml::to_vec(self)?)?;
        Ok(())
    }

    pub fn get_team_name(&self) -> &str {
        &self.team_name
    }

    pub fn get_clip_server_port(&self) -> &SocketAddr {
        &self.clip_server_port
    }

    pub fn get_web_server_port(&self) -> &SocketAddr {
        &self.web_server_port
    }

    pub fn get_discovery_port(&self) -> &SocketAddr {
        &self.discovery_port
    }

    pub fn get_output_directory(&self) -> &Path {
        &self.output_directory
    }

    pub fn get_database_name(&self) -> &str {
        &self.database_name
    }

}

impl Default for ServerConfiguration {
    fn default() -> Self {
        ServerConfiguration {
            team_name: String::from("TEAM_NAME"),

            clip_server_port: net::SocketAddr::new(net::IpAddr::from(net::Ipv4Addr::new(127, 0, 0, 1)), 8000),
            web_server_port: net::SocketAddr::new(net::IpAddr::from(net::Ipv4Addr::new(127, 0, 0, 1)), 8080),
            discovery_port: net::SocketAddr::new(net::IpAddr::from(net::Ipv4Addr::new(127, 0, 0, 1)), 9000),

            output_directory: PathBuf::from("./out/"),
            database_name: String::from("primary_database.db"),
        }
    }
}