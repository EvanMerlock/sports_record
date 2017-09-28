extern crate sports_record;

use std::net::{SocketAddr, AddrParseError};
use std::str::FromStr;
use std::env;
use std::path::Path;
use std::fs::File;

use sports_record::config::client_configuration::ClientConfiguration;
use sports_record::client::ClientError;
use sports_record::client::client_struct::{Client};

fn main() {
    let result = run_client();
    println!("Client Closing, Result: {:?}", result);
}

fn run_client() -> Result<(), ClientError> {
    let configuration_location = env::var("SR_RECORDING_LOCATION")?;
    let config_path = Path::new(&configuration_location);
    let client_config = 
        if config_path.exists() {
            let fs = File::open(config_path)?;
            ClientConfiguration::from(fs)?
        } else {
            let fs = File::create(config_path)?;
            let cfg = ClientConfiguration::default();
            cfg.write_to(fs)?;
            cfg
        };

    let server_addr = try!(string_to_socket("127.0.0.1:8000"));
    let mut client = try!(Client::new(client_config.get_name(), server_addr));

    let _ = client.stream_handler(client_config.get_camera_settings().clone());

    Ok(())
}

fn string_to_socket(ip: &str) -> Result<SocketAddr, AddrParseError> {
    SocketAddr::from_str(ip)
}
