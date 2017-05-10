use unsafe_code::vid_processing::{send_video, write_to_stream};
use messenger_plus::stream::DualMessenger;

use std::net::{SocketAddr, TcpStream, TcpListener};
use client::errors::ClientError;
use std::io::{Write, Read};
use std::thread;
use std::sync::mpsc::{channel, Sender, Receiver};

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
        let mut dual_channel: DualMessenger<TcpStream> = DualMessenger::new(String::from("--"), String::from("boundary"), String::from("endboundary"), &mut self.stream);

        let mut stream_open = true;
        
        while stream_open {
            let results = dual_channel.read_next_message();
            match results {
                None => {
                    println!("server eos");
                    let stream_open = false;
                },
                Some(v) => {
                    // check to see if anymore instructions have been sent through
                    let curr_data = String::from_utf8(v);
                    match curr_data {
                        Ok(s) => {
                            if s == "START" {
                                let (tx, rx) = channel();
                                thread::spawn(|| {
                                    println!("Sent video: {:?}", send_video(tx));
                                });
                                for item in rx {
                                    let _ = write_to_stream(item, &mut dual_channel);
                                }
                            }
                        },
                        Err(e) => {
                            panic!("something went terribly wrong! {:?}", e);
                        }
                    }
                },
            }
        }

        Ok(())
    }
}