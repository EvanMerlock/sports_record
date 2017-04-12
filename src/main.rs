extern crate sports_record;

use std::net::{SocketAddr, AddrParseError};
use std::str::FromStr;
use std::path::Path;
use std::io::{stdin, BufRead};

use sports_record::server::ServerError;
use sports_record::server::RecordingServer;
use sports_record::server::client_handling::{RecordingInstructions, ClientIPInformation};

fn main() {
    let _ = run_server();
}

fn run_server() -> Result<(), ServerError> {
    let sock_tuple = try!(strs_to_socket_tuple("127.0.0.1:8000", "127.0.0.1:8080"));
    let server = try!(RecordingServer::new(sock_tuple, &Path::new("output/maindb.db")));

    let messenger = server.get_instruction_sender();
    let ip_messenger = server.get_ip_sender();

    server.start_handling_requests();

    let stdin = stdin();
    let line_lock = stdin.lock();

    for line in line_lock.lines() {
        let unwrapped_line = try!(line);
        println!("Recv: {}", unwrapped_line);
        if unwrapped_line == "START" {
            println!("RUNNING START Routine");
            let _ = messenger.send(RecordingInstructions::StartRecording(1));
        } else if unwrapped_line == "STOP" {
            println!("RUNNING STOP Routine");
            let _ = messenger.send(RecordingInstructions::StopRecording);
        } else if unwrapped_line == "CLEAN" {
            println!("RUNNING CLEAN Routine");
            let _ = messenger.send(RecordingInstructions::Cleanup);
        } else if unwrapped_line.starts_with("REMOVE") {
            let whitespace_iter = unwrapped_line.split_whitespace();
            let whitespace_ip_iter = whitespace_iter.filter_map(|x| {
                match string_to_socket(x) {
                    Ok(e) => Some(e),
                    Err(_) => None, 
                }
            });

            for ip in whitespace_ip_iter {
                println!("Removing: {:?}", ip);
                let _ = ip_messenger.send(ClientIPInformation::Remove(ip));
            }
        }
    }

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
