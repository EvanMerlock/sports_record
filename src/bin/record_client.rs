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

    let mut client = Client::new(client_config.get_name(), (SocketAddr::from_str("127.0.0.1:8000")?, SocketAddr::from_str("127.0.0.1:4000")?))?;

    let sender = client.get_web_handler_ref().get_sender();

    let _ = client.stream_handler(client_config.get_camera_settings().clone(), sender);

    Ok(())
}
