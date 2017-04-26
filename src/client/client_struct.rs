use unsafe_code::vid_processing::send_video;

use std::net::{SocketAddr, TcpStream, TcpListener};
use client::errors::ClientError;
use std::io::{Write, Read};
use std::thread;

#[derive(Debug)]
pub struct Client {
    server_ip: SocketAddr,
    stream: TcpStream,
}

impl Client {
    pub fn new(sock: SocketAddr) -> Result<Client, ClientError> {
        let mut stream = try!(TcpStream::connect(sock));

        Ok(Client { server_ip: sock, stream: stream })
    }

    pub fn stream_handler(&mut self) -> Result<(), ClientError> {
        let mut buffer = [0; 1];
        let mut large_buffer: Vec<u8> = Vec::new();

        let mut stream_open = true;
        
        while stream_open {
            let results = self.stream.read(&mut buffer);
            match results {
                Ok(i) if i == 0 => {
                    println!("server eos");
                    let stream_open = false;
                },
                Ok(..) => {
                    large_buffer.push(buffer[0]);
                    // first check to see if anymore instructions have been sent through
                    if large_buffer.len() > 0 {
                        let curr_data = String::from_utf8(large_buffer.clone());
                        match curr_data {
                            Ok(s) => {
                                if s == "--STRT" {
                                    println!("Sent video: {:?}", send_video(&mut self.stream));
                                }
                            },
                            Err(e) => {
                                panic!("something went terribly wrong! {:?}", e);
                            }
                        }
                    }
                },
                Err(e) => {
                    panic!("something went terribly wrong! {:?}", e);
                }
            }
        }

        Ok(())
    }
}