use std::net::{SocketAddr, TcpStream};
use std::thread;
use std::thread::JoinHandle;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{channel, Sender};
use std::cell::Cell;

use messenger_plus::stream::DualMessenger;
use messenger_plus::stream;
use config::client_configuration::CameraConfiguration;
use client::errors::ClientError;
use client::{ClientStatusFlag, send_video};
use client::web::WebHandler;
use networking::NetworkPacket;

use unsafe_code::UnsafeError;

pub struct Client {
    name: String,
    stream: TcpStream,
    http_server: WebHandler,
}

impl Client {
    pub fn new(name: &str, socks: (SocketAddr, SocketAddr, SocketAddr)) -> Result<Client, ClientError> {
        let stream = try!(TcpStream::connect(socks.0));
        let wh_tuple = WebHandler::new((socks.1, socks.2))?;

        Ok(Client { name: String::from(name), http_server: wh_tuple, stream: stream })
    }

    pub fn stream_handler(&mut self, camera_config: CameraConfiguration, arc_sender: Sender<Arc<Vec<u8>>>) -> Result<(), ClientError> {
        let read_stream = try!(self.stream.try_clone());
        let write_stream = try!(self.stream.try_clone());
        let mut read_channel: DualMessenger<TcpStream> = DualMessenger::new(String::from("--"), String::from("boundary"), String::from("endboundary"), read_stream);
        let write_channel: DualMessenger<TcpStream> = DualMessenger::new(String::from("--"), String::from("boundary"), String::from("endboundary"), write_stream);
        let mut video_processing = ClientVideoThreadHandler::new(write_channel, camera_config, arc_sender);

        let mut stream_open = true;
        
        while stream_open {
            let results = read_channel.read_next_message();
            match results {
                Err(ref e) if e == &stream::Error::from(stream::ErrorKind::BufferEmpty) => {
                    println!("Server EOS");
                    video_processing.server_disconnect();
                    stream_open = false;
                },
                Ok(v) => {
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
                Err(e) => return Err(ClientError::from(UnsafeError::from(e))),
            }            
        }

        Ok(())
    }

    pub fn get_web_handler_ref(&self) -> &WebHandler {
        &self.http_server
    }
}

struct ClientVideoThreadHandler {
    send_video_handle: Cell<JoinHandle<()>>,
    write_video_handle: Cell<JoinHandle<()>>,
    video_tunnel: Sender<ClientStatusFlag>,
}

impl ClientVideoThreadHandler {
    fn new<'a>(mut write_channel: DualMessenger<TcpStream>, camera_config: CameraConfiguration, jpeg_sender: Sender<Arc<Vec<u8>>>) -> ClientVideoThreadHandler {
        let (instr_tx, instr_rx) = channel();
        let (tx, rx) = channel::<NetworkPacket>();
        let send_video_handle = thread::Builder::new().name("send_video_thread".to_string()).spawn(|| {
            println!("Send Video Completion Status: {:?}", send_video(camera_config, instr_rx, tx, jpeg_sender));
        }).unwrap();
        let write_video_handle = thread::Builder::new().name("write_video_thread".to_string()).spawn(move || {
            for item in rx {
                let _ = item.write_to(&mut write_channel);
            }
        }).unwrap();
        ClientVideoThreadHandler {
            send_video_handle: Cell::new(send_video_handle),
            write_video_handle: Cell::new(write_video_handle),
            video_tunnel: instr_tx,
        }
    }

    fn start(&self) {
        let _ = self.video_tunnel.send(ClientStatusFlag::StartRecording);
    }

    fn stop(&self) {
        let _ = self.video_tunnel.send(ClientStatusFlag::StopRecording);
    }

    fn server_disconnect(&self) {
        let _ = self.video_tunnel.send(ClientStatusFlag::ServerQuit);
    }
}