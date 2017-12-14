#[macro_use]
extern crate serde_derive;

extern crate iron;
extern crate router;
extern crate rusqlite;
extern crate ffmpeg_sys;
extern crate uuid;
extern crate serde;
extern crate serde_json;
extern crate messenger_plus;
extern crate time;
extern crate libc;
extern crate toml;
extern crate websocket;
extern crate liquid;
extern crate base64;
extern crate rand;
extern crate ffmpeg_common;

use std::net::{SocketAddr};
use std::str::FromStr;
use std::env;
use std::path::Path;
use std::fs::File;

mod client;
use client::{ClientError, ClientConfiguration};
use client::client_struct::{Client};

fn main() {
    let result = run_client();
    println!("Client Closing, Result: {:?}", result);
}

fn run_client() -> Result<(), ClientError> {
    let configuration_location: String = match env::var("SR_CLIENTCONF_LOC") {
        Ok(item) => item,
        Err(ref e) if e == &env::VarError::NotPresent => String::from("sr_client_config.toml"),
        Err(e) => return Err(ClientError::from(e)),
    };    
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

    let mut client = Client::new(client_config.get_name(), (SocketAddr::from_str("127.0.0.1:8000")?, SocketAddr::from_str("127.0.0.1:4000")?, SocketAddr::from_str("127.0.0.1:8070")?))?;

    let sender = client.get_web_handler_ref().get_sender();

    let _ = client.stream_handler(client_config.get_camera_settings().clone(), sender);

    Ok(())
}
