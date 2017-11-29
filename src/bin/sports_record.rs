extern crate sports_record;

use std::env;
use std::net::{SocketAddr, AddrParseError};
use std::str::FromStr;
use std::path::Path;
use std::fs::File;
use std::io::{stdin, BufRead};

use sports_record::config::server_configuration::ServerConfiguration;
use sports_record::server::ServerError;
use sports_record::server::RecordingServer;

fn main() {
    println!("{:?}", run_server());
}

fn run_server() -> Result<(), ServerError> {

    let configuration_location: String = match env::var("SR_SERVERCONF_LOC") {
        Ok(item) => item,
        Err(ref e) if e == &env::VarError::NotPresent => String::from("sr_server_config.toml"),
        Err(e) => return Err(ServerError::from(e)),
    };
    let config_path = Path::new(&configuration_location);
    let server_config: ServerConfiguration = 
        if config_path.exists() {
            let fs = File::open(config_path)?;
            ServerConfiguration::from(fs)?
        } else {
            let fs = File::create(config_path)?;
            let cfg = ServerConfiguration::default();
            cfg.write_to(fs)?;
            cfg
        }; 
    let server = RecordingServer::new(server_config)?;

    let mut messenger = server.get_client_handler();

    server.start_handling_requests();

    let stdin = stdin();
    let line_lock = stdin.lock();


    for line in line_lock.lines() {
        let unwrapped_line = try!(line);
        if unwrapped_line == "START" {
            println!("Starting recording");
            let _ = messenger.start_recording();
        } else if unwrapped_line == "STOP" {
            println!("Stopping recording");
            let _ = messenger.stop_recording();
        } else if unwrapped_line == "CLEAN" {
            println!("Stopping server");
            let _ = messenger.clean_up();
        } else if unwrapped_line.starts_with("REMOVE") {
            let whitespace_iter = unwrapped_line.split_whitespace();
            let whitespace_ip_iter = whitespace_iter.filter_map(|x| {
                match string_to_socket(x) {
                    Ok(e) => Some(e),
                    Err(_) => None, 
                }
            });

            for ip in whitespace_ip_iter {
                println!("Removing {:?}", ip);
                messenger.remove_client(ip);
            }
        }
    }

    Ok(())

}

fn string_to_socket(ip: &str) -> Result<SocketAddr, AddrParseError> {
    SocketAddr::from_str(ip)
}
