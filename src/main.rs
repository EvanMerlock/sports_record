extern crate iron;
extern crate router;
extern crate rusqlite;

mod server;
mod client;

use std::env;
use std::net::{SocketAddr, AddrParseError};
use std::str::FromStr;
use std::path::Path;
use std::io::{stdin, BufRead};

use server::errors::ServerError;
use server::messenger::{CurrentServerStatus, Messenger};
use client::errors::ClientError;
use server::client_handling::client_stream::RecordingInstructions;

fn main() {

    let mut is_client = false;

    for arg in env::args() {
        println!("Argument: {}", arg);
        if arg == "-client" {
            is_client = true;
        }
    }

    if !is_client {
        let _ = run_server();
    } else {
        let _ = run_client();
    }
}

fn run_server() -> Result<(), ServerError> {
    let sock_tuple = try!(strs_to_socket_tuple("127.0.0.1:8000", "127.0.0.1:8080"));
    let server = try!(server::recording_server::RecordingServer::new(sock_tuple, &Path::new("output/maindb.db")));

    let messenger = server.get_instruction_sender();

    server.start_handling_requests();

    let stdin = stdin();
    let line_lock = stdin.lock();

    for line in line_lock.lines() {
        let unwrapped_line = try!(line);
        println!("Recv: {}", unwrapped_line);
        if unwrapped_line == "START" {
            {
                println!("RUNNING START Routine");
                messenger.send(RecordingInstructions::StartRecording(1));
            }
        } else if unwrapped_line == "STOP" {
            {
                println!("RUNNING STOP Routine");
                messenger.send(RecordingInstructions::StopRecording);
            }
        } else if unwrapped_line == "CLEAN" {
            println!("RUNNING CLEAN Routine");
            messenger.send(RecordingInstructions::Cleanup);
        }
    }

    Ok(())

}

fn run_client() -> Result<(), ClientError> {
    let sock_addr = try!(strs_to_socket_tuple("127.0.0.1:8000", "127.0.0.1:9000"));
    let client = try!(client::client_struct::Client::new(sock_addr));

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
