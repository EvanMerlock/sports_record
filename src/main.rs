extern crate sports_record;

use std::net::{SocketAddr, AddrParseError};
use std::str::FromStr;
use std::path::Path;
use std::io::{stdin, BufRead};

use sports_record::server::ServerError;
use sports_record::server::RecordingServer;

fn main() {
    println!("{:?}", run_server());
}

fn run_server() -> Result<(), ServerError> {
    let sock_tuple = try!(strs_to_socket_tuple("127.0.0.1:8000", "127.0.0.1:8080"));
    let server = try!(RecordingServer::new(sock_tuple, &Path::new("out/maindb.db")));

    let messenger = server.get_client_handler();

    server.start_handling_requests();

    let stdin = stdin();
    let line_lock = stdin.lock();

    let mut current_clip = 1;

    for line in line_lock.lines() {
        let unwrapped_line = try!(line);
        if unwrapped_line == "START" {
            println!("Starting recording");
            let _ = messenger.start_recording(current_clip);
        } else if unwrapped_line == "STOP" {
            println!("Stopping recording");
            let _ = messenger.stop_recording();
            current_clip = current_clip + 1;
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

fn strs_to_socket_tuple(ip_one: &str, ip_two: &str) -> Result<(SocketAddr, SocketAddr), AddrParseError> {
    let socket_one = try!(string_to_socket(ip_one));
    let socket_two = try!(string_to_socket(ip_two));

    Ok((socket_one, socket_two))
}

fn string_to_socket(ip: &str) -> Result<SocketAddr, AddrParseError> {
    SocketAddr::from_str(ip)
}
