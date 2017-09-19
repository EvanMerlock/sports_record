use std::net::{SocketAddr, TcpStream, Shutdown};
use std::thread;
use std::thread::JoinHandle;
use std::sync::mpsc::{channel, Sender, Receiver};
use std::cell::Cell;

use messenger_plus::stream::DualMessenger;
use client::errors::ClientError;
use client::{ClientStatusFlag, send_video};
use networking::NetworkPacket;

use unsafe_code::UnsafeError;

#[derive(Debug)]
pub struct Client {
    server_ip: SocketAddr,
    stream: TcpStream,
}

impl Client {
    pub fn new(sock: SocketAddr) -> Result<Client, ClientError> {
        let stream = try!(TcpStream::connect(sock));

        Ok(Client { server_ip: sock, stream: stream })
    }

    pub fn stream_handler(&mut self) -> Result<(), ClientError> {
        let mut read_stream = try!(self.stream.try_clone());
        let mut write_stream = try!(self.stream.try_clone());
        let mut read_channel: DualMessenger<TcpStream> = DualMessenger::new(String::from("--"), String::from("boundary"), String::from("endboundary"), read_stream);
        let mut write_channel: DualMessenger<TcpStream> = DualMessenger::new(String::from("--"), String::from("boundary"), String::from("endboundary"), write_stream);
        let mut video_processing = ClientVideoThreadHandler::new(write_channel);

        let mut stream_open = true;
        
        while stream_open {
            let results = read_channel.read_next_message();
            match results {
                None => {
                    println!("Server EOS");
                    stream_open = false;
                },
                Some(v) => {
                    // check to see if anymore instructions have been sent through
                    let curr_data = String::from_utf8(v);
                    match curr_data {
                        Ok(s) => {
                            match s.as_ref() {
                                "START" => {
                                    println!("Starting Recording");
                                    video_processing.start();
                                },
                                "STOP" => {
                                    println!("Stopping Recording");
                                    video_processing.stop();
                                },
                                _ => println!("Received Unsupported Instruction"),
                            }
                        },
                        Err(e) => println!("{}", e),
                    }
                },
            }            
        }

        Ok(())
    }
}

struct ClientVideoThreadHandler {
    send_video_handle: Cell<JoinHandle<()>>,
    write_video_handle: Cell<JoinHandle<()>>,
    video_tunnel: Sender<ClientStatusFlag>,
}

impl ClientVideoThreadHandler {
    fn new(mut write_channel: DualMessenger<TcpStream>) -> ClientVideoThreadHandler {
        let (instr_tx, instr_rx) = channel();
        let (tx, rx) = channel::<NetworkPacket>();
        let send_video_handle = thread::spawn(|| {
            println!("Send Video Completion Status: {:?}", send_video(instr_rx, tx));
        });
        let write_video_handle = thread::spawn(move || {
            for item in rx {
                let _ = item.write_to(&mut write_channel);
            }
        });
        ClientVideoThreadHandler {
            send_video_handle: Cell::new(send_video_handle),
            write_video_handle: Cell::new(write_video_handle),
            video_tunnel: instr_tx,
        }
    }

    fn start(&mut self) {
        self.video_tunnel.send(ClientStatusFlag::StartRecording);
    }

    fn stop(&mut self) {
        self.video_tunnel.send(ClientStatusFlag::StopRecording);
    }
}