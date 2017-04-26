use std::io::prelude::*;
use std::net::{TcpStream, Shutdown};
use std::str;
use std::sync::mpsc::Sender;

use server::errors::ServerError;
use server::client_handling::{ClientIPInformation};

#[derive(Debug)]
enum IOCode {
    NewClient,
    ClientPort(u16)
}

pub fn stream_handler(mut stream: TcpStream, ip_list_sender: Sender<ClientIPInformation>) -> Result<i32, ServerError> {

    println!("Received client");
    let mut completed_stream = false;
    
    let mut client_port: u16 = 0;

    while !completed_stream {
        let mut buffer = [0; 8];
        let read_results = stream.read(&mut buffer);

        match read_results {
            Ok(i) if i == 0 => { 
                println!("Reached end of stream");
                completed_stream = true;
             }
            Ok(..) => { 
                print!("Received data: {:?} | ", buffer);

                match try!(find_init_code(&buffer)) {
                    Some(IOCode::NewClient) => {
                        let mut client_addr = try!(stream.peer_addr());

                        if client_port == 0 {
                            let _ = stream.write(b"Choose a port by sending it over with no formatting\n");
                            continue;
                        }

                        client_addr.set_port(client_port);
                        println!("New Client: {:?} ", client_addr);
                        let _ = ip_list_sender.send(ClientIPInformation::Add(client_addr));
                        let _ = stream.shutdown(Shutdown::Both);
                        continue;
                    },
                    Some(IOCode::ClientPort(r)) => {
                        println!("Setting client port: {:?}", r);
                        client_port = r.clone();
                        continue;
                    }
                    None => {
                        println!("Some other data recieved. Ignoring it, and continuing waiting for an init code");
                        continue;
                    }
                }
            }
            Err(e) => println!("Err: {}", e)
        }

    }

    let _ = stream.shutdown(Shutdown::Both);
    Ok(0)

}

fn find_init_code(buf: &[u8]) -> Result<Option<IOCode>, ServerError> {
    let utf8_str = try!(str::from_utf8(buf));

    let owned = utf8_str.to_owned();
    let trimmed = owned.trim_matches(|x: char| x == '\n' || x == '\u{0}' );

    if trimmed == "INTC_01" {
        print!("Code is INTC_01 | ");
        return Ok(Some(IOCode::NewClient));
    }

    let trimmed_int = trimmed.parse::<u16>();

    match trimmed_int {
        Ok(r) => Ok(Some(IOCode::ClientPort(r))),
        _ => Ok(None)
    }

}