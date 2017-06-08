extern crate sports_record;

use std::net::{SocketAddr, AddrParseError};
use std::str::FromStr;

use sports_record::client::ClientError;
use sports_record::client::client_struct::{Client};

fn main() {
    let _ = run_client();
}

fn run_client() -> Result<(), ClientError> {
    let sock_addr = try!(string_to_socket("127.0.0.1:8000"));
    let mut client = try!(Client::new(sock_addr));

    let _ = client.stream_handler();

    Ok(())
}

fn string_to_socket(ip: &str) -> Result<SocketAddr, AddrParseError> {
    SocketAddr::from_str(ip)
}
