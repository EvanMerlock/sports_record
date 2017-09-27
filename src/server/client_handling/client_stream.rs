use std::net::{TcpStream, SocketAddr, Shutdown};
use std::thread;
use std::thread::JoinHandle;
use std::result::Result;
use std::sync::mpsc::{Sender, Receiver, channel};
use std::io::Write;
use std::cell::Cell;

use std::ffi::CString;

use server::ServerError;
use config::stream_config::StreamConfiguration;

use unsafe_code::format::{FormatContext, OutputContext};
use unsafe_code::{EncodingCodecContext, Rational, CodecId, Packet, UnsafeError, UnsafeErrorKind};
use networking::NetworkPacket;

use uuid::Uuid;
use serde_json;
use messenger_plus::stream::{DualMessenger};
use messenger_plus::stream;

use ffmpeg_sys::*;

pub struct ClientStream {
    current_clients: Vec<ClientThreadInformation>,
}

pub struct ClientThreadInformation {
    socket_addr: SocketAddr,
    thread_handle: JoinHandle<()>,
    thread_channel: Sender<RecordingInstructions>,
}

#[derive(Debug)]
pub enum ClientIPInformation {
    Add(TcpStream),
    Remove(SocketAddr),
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum RecordingInstructions {
    StartRecording(u32),
    StopRecording,
    Cleanup,
}

impl ClientThreadInformation {
    pub fn new(sock: SocketAddr, tcp_stream: TcpStream) -> ClientThreadInformation {
        let (send, recv) = channel();
        let thread_handle = thread::spawn(move || {
            let val = client_write_handler(tcp_stream, recv);
            println!("{:?}", val);
        });
        ClientThreadInformation { socket_addr: sock, thread_handle: thread_handle, thread_channel: send }
    }
}

impl Drop for ClientThreadInformation {
    fn drop(&mut self) {
        let _ = self.thread_channel.send(RecordingInstructions::Cleanup);
    }
}

impl ClientStream {

    pub fn new() -> Result<ClientStream, ServerError> {
        let stream = ClientStream {
            current_clients: vec![],
        };
        Ok(stream)
    }

    pub fn add_client(&mut self, info: TcpStream) -> Result<(), ServerError> {
        let socket_addr = try!(info.peer_addr());
        self.current_clients.push(ClientThreadInformation::new(socket_addr, info));
        Ok(())
    }

    pub fn remove_client(&mut self, info: SocketAddr) {
        let mut new_thread_list: Vec<ClientThreadInformation> = Vec::new();

        for thread_info in self.current_clients.drain(0..) {
            if !(thread_info.socket_addr == info) {
                new_thread_list.push(thread_info);
            } else {
                let _ = thread_info.thread_channel.send(RecordingInstructions::Cleanup);
            }
        }
        self.current_clients.append(&mut new_thread_list);
    }

    pub fn start_recording(&self, num: u32) {
        self.send_command(RecordingInstructions::StartRecording(num));
    }

    pub fn stop_recording(&self) {
        self.send_command(RecordingInstructions::StopRecording);
    }

    pub fn clean_up(&mut self) {
        self.send_command(RecordingInstructions::Cleanup);
        self.current_clients.clear();
    }

    fn send_command(&self, current_command: RecordingInstructions) {
        for item in &self.current_clients {
            let _ = item.thread_channel.send(current_command);
        }
    }

}

fn client_write_handler(stream: TcpStream, recv: Receiver<RecordingInstructions>) -> Result<(), ServerError> {

    let mut currently_cleaning = false;

    let write_stream = try!(stream.try_clone());
    let read_stream = try!(stream.try_clone());
    
    let mut read_channel: DualMessenger<TcpStream> = DualMessenger::new(String::from("--"), String::from("boundary"), String::from("endboundary"), read_stream);
    let mut write_channel: DualMessenger<TcpStream> = DualMessenger::new(String::from("--"), String::from("boundary"), String::from("endboundary"), write_stream);

    println!("Attempting to retrieve stream configuration from client {}", stream.peer_addr()?);
    let results = read_channel.read_next_message().map_err(|x| UnsafeError::from(x))?;
    let stream_config = serde_json::from_slice::<NetworkPacket>(results.as_ref()).map_err(|x| UnsafeError::from(x))?;
    let unwrapped_config = match stream_config {
        NetworkPacket::JSONPayload(e) => e,
        _ => return Err(ServerError::from(UnsafeError::new(UnsafeErrorKind::OpenInput(1000)))),
    };
    println!("Retreived stream configuration from client {}", stream.peer_addr()?);
    println!("{:?}", unwrapped_config);

    let mut stcth = ServerToClientThreadHandler::new(unwrapped_config, read_channel);

    while !currently_cleaning {
        loop {
            let curr_instruction = recv.recv().unwrap();
            match curr_instruction {
                RecordingInstructions::StartRecording(i) => {
                    let _ = write_channel.write(b"START");
                    stcth.start();
                },
                RecordingInstructions::StopRecording => {
                    let _ = write_channel.write(b"STOP");
                    stcth.stop(DualMessenger::new(String::from("--"), String::from("boundary"), String::from("endboundary"), stream.try_clone()?));
                }
                RecordingInstructions::Cleanup => {
                    currently_cleaning = true;
                    break;
                }
            }
        }
        println!("In clean-up loop");
    }
    println!("Cleaning up");
    let _ = stream.shutdown(Shutdown::Both);
    Ok(())
}

fn receive_video(conf: StreamConfiguration, mut read_channel: DualMessenger<TcpStream>, instr_recv: Receiver<TranslatedRecordingInstructions>) -> Result<i32, ServerError> {

    let _ = try!(instr_recv.recv());

    let mut frames_read = 0;
    let mut completed_stream = false;

    let file_path: String = String::from("output/video_") + &Uuid::new_v4().simple().to_string() + ".mp4";
    let mut format_context: OutputContext = FormatContext::new_output(CString::new(file_path.as_str()).unwrap());
    println!("Created output context");
    let encoding_context = try!(EncodingCodecContext::create_encoding_context(CodecId::from(AV_CODEC_ID_H264), conf.height, conf.width, conf.time_base, conf.gop_size, conf.max_b_frames));
    let pkt_stream = format_context.create_stream(encoding_context);

    let stream_index = pkt_stream.index;
                
    println!("Created output video stream");
    try!(format_context.open_video_file(file_path.as_ref()));
    println!("Opened video file: {}", file_path.as_str());
    try!(format_context.write_video_header());
    println!("Wrote video header");

    let stream_timebase = Rational::from(pkt_stream.time_base);
    while !completed_stream {
        let results = read_channel.read_next_message();

        match results {
            Err(ref e) if e == &stream::Error::from(stream::ErrorKind::BufferEmpty) => {
                println!("Read {} messages from stream, now reached EOS.", frames_read);
                completed_stream = true;
            }
            Ok(v) => {
                frames_read = frames_read + 1;
                let data_packet_attempt = serde_json::from_slice::<NetworkPacket>(v.as_slice()).map_err(|x| UnsafeError::from(x));
                match data_packet_attempt {
                    Ok(network_packet) => {
                        match network_packet {
                            NetworkPacket::PacketStream(pkts) => {
                                for mut pkt in pkts.into_iter().map(|x| Packet::from(x)) {
                                    println!("Recieved packet from client with pts {}", pkt.pts);
                                    pkt.rescale_to(Rational::new(1,30), stream_timebase);
                                    //pkt.dts = pkt.pts;
                                    let _ = format_context.write_video_frame(stream_index, pkt)?;
                                }
                            },
                            NetworkPacket::PayloadEnd => {
                                println!("Received EOP Indicator");
                                completed_stream = true;
                                continue;
                            },
                            _ => eprintln!("Unexpected Network Packet Type!"),
                        }
                    },
                    Err(e) => eprintln!("{:?}, {:?}", e, v.as_slice()),
                }
            },
            Err(e) => {
                return Err(ServerError::from(UnsafeError::from(e)));
            }
        }
    }
    println!("Current read ended, {} frames read.", frames_read);
    try!(format_context.write_null_video_frame());
    try!(format_context.write_video_trailer());
    println!("Wrote video trailer and null video frame");
    Ok(frames_read)
}

#[derive(Debug)]
enum TranslatedRecordingInstructions {
    Start,
    Stop,
}

struct ServerToClientThreadHandler {
    rec_vid_thread: Cell<JoinHandle<Result<i32, ServerError>>>,
    instr_tun: Sender<TranslatedRecordingInstructions>,
    conf: StreamConfiguration,
}

impl ServerToClientThreadHandler {
    fn new(conf: StreamConfiguration, read_channel: DualMessenger<TcpStream>) -> ServerToClientThreadHandler {
        let (send, recv) = channel();
        let rec_vid_thread = thread::spawn(move || {
            receive_video(conf, read_channel, recv)
        });
        ServerToClientThreadHandler {
            rec_vid_thread: Cell::new(rec_vid_thread),
            instr_tun: send,
            conf: conf,
        }
    }

    fn start(&mut self) {
        let _ = self.instr_tun.send(TranslatedRecordingInstructions::Start);
    }

    fn stop(&mut self, read_channel: DualMessenger<TcpStream>) {
        let (instr_tx, instr_rx) = channel();
        let pass_conf = self.conf.clone();
        let rec_vid_thread = thread::spawn(move || {
            receive_video(pass_conf, read_channel, instr_rx)
        });
        self.instr_tun = instr_tx;
        println!("{:?}", self.rec_vid_thread.replace(rec_vid_thread).join());
    }
}