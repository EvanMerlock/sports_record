extern crate sports_record;

use std::net::{SocketAddr, AddrParseError};
use std::str::FromStr;

use sports_record::client::ClientError;
use sports_record::client::client_struct::Client;

fn main() {
    let _ = run_client();
}

fn run_client() -> Result<(), ClientError> {
    let sock_addr = try!(strs_to_socket_tuple("127.0.0.1:8000", "127.0.0.1:9000"));
    let client = try!(Client::new(sock_addr));

    client.handle();

    Ok(())
}

fn strs_to_socket_tuple(ip_one: &str, ip_two: &str) -> Result<(SocketAddr, SocketAddr), AddrParseError> {
    let socket_one = try!(string_to_socket(ip_one));
    let socket_two = try!(string_to_socket(ip_two));

    Ok((socket_one, socket_two))
}

fn string_to_socket(ip: &str) -> Result<SocketAddr, AddrParseError> {
    SocketAddr::from_str(ip)
}
