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

use unsafe_code::vid_processing;
use unsafe_code::format::{FormatContext, InputContext, open_video_file, write_video_frame, write_video_header, write_video_trailer, write_null_video_frame};
use unsafe_code::{Rational, CodecId, Packet, DataPacket};
use unsafe_code::format::OutputContext;

use uuid::Uuid;
use serde_json;
use serde_cbor;
use messenger_plus::stream::{DualMessenger};

use ffmpeg_sys::*;

pub struct ClientStream {
    client_handler_channel: Sender<ClientIPInformation>,
    client_handler_lock: JoinHandle<()>,

    instruction_sender: Sender<RecordingInstructions>,
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
    pub fn new(sock: SocketAddr, thread_handle: JoinHandle<()>, channel: Sender<RecordingInstructions>) -> ClientThreadInformation {
        ClientThreadInformation { socket_addr: sock, thread_handle: thread_handle, thread_channel: channel }
    }
}

impl Drop for ClientThreadInformation {
    fn drop(&mut self) {
        let _ = self.thread_channel.send(RecordingInstructions::Cleanup);
    }
}

impl ClientStream {

    pub fn new() -> Result<ClientStream, ServerError> {
        let (client_ip_send, client_ip_recv) = channel();
        let (instruct_send, instruct_recv) = channel();

        let client_handler_channel = thread::spawn(move || {
            main_client_handler(client_ip_recv, instruct_recv);
        });

        let stream = ClientStream {
            client_handler_channel: client_ip_send,
            client_handler_lock: client_handler_channel,

            instruction_sender: instruct_send,
        };

        Ok(stream)
        
    }

    pub fn get_instruction_sender(&self) -> Sender<RecordingInstructions> {
        self.instruction_sender.clone()
    }

    pub fn get_ip_information_sender(&self) -> Sender<ClientIPInformation> {
        self.client_handler_channel.clone()
    }

}

fn main_client_handler(ip_channel: Receiver<ClientIPInformation>, instruction_receiver: Receiver<RecordingInstructions>) {

    let mut thread_list: Vec<ClientThreadInformation> = Vec::new();
    
    let mut end_exec = false;

    while !end_exec {
        let current_instruction = instruction_receiver.recv().unwrap();

        match current_instruction {
            RecordingInstructions::Cleanup => {
                for item in &thread_list {
                    let _ = item.thread_channel.send(current_instruction);
                }

                end_exec = true;
            },
            _ => {
                let _ = handle_client_changes(&mut thread_list, &ip_channel);

                for item in &thread_list {
                    let _ = item.thread_channel.send(current_instruction);
                }
            }
        }
    }

}

fn handle_client_changes(thread_list: &mut Vec<ClientThreadInformation>, ip_channel: &Receiver<ClientIPInformation>) -> Result<(), ServerError> {
    for ip_info in ip_channel.try_iter() {
        match ip_info {
            ClientIPInformation::Add(i) => {
                let (send, recv) = channel();
                let socket_addr = try!(i.peer_addr());
                let temp_thread_handle = thread::spawn(move || {
                    let val = individual_client_handler(i, recv);
                    println!("{:?}", val);
                });
                thread_list.push(ClientThreadInformation::new(socket_addr, temp_thread_handle, send));
            }, 
            ClientIPInformation::Remove(i) => {
                let mut new_thread_list: Vec<ClientThreadInformation> = Vec::new();

                for thread_info in thread_list.drain(0..) {
                    if !(thread_info.socket_addr == i) {
                        new_thread_list.push(thread_info);
                    } else {
                        let _ = thread_info.thread_channel.send(RecordingInstructions::Cleanup);
                    }
                }
                thread_list.append(&mut new_thread_list);
            },
        }
    }
    Ok(())
}

fn individual_client_handler(mut stream: TcpStream, recv: Receiver<RecordingInstructions>) -> Result<(), ServerError> {

    let mut currently_cleaning = false;

    let mut write_stream = try!(stream.try_clone());
    let mut read_stream = try!(stream.try_clone());
    
    let mut read_channel: DualMessenger<TcpStream> = DualMessenger::new(String::from("--"), String::from("boundary"), String::from("endboundary"), read_stream);
    let mut write_channel: DualMessenger<TcpStream> = DualMessenger::new(String::from("--"), String::from("boundary"), String::from("endboundary"), write_stream);

    let mut stcth = ServerToClientThreadHandler::new(read_channel);

    while !currently_cleaning {
        loop {
            let curr_instruction = recv.recv().unwrap();
            let mut frames_read = 0;
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

fn receive_video(mut read_channel: DualMessenger<TcpStream>, instr_recv: Receiver<TranslatedRecordingInstructions>) -> Result<i32, ServerError> {

    let _ = try!(instr_recv.recv());

    let mut frames_read = 0;
    let mut completed_stream = false;

    println!("Attempting to retrieve stream configuration");
    let results = read_channel.read_next_message();
    let stream_config = results.map(|x| {
        serde_json::from_slice::<StreamConfiguration>(x.as_ref())
    }).expect("failure to collect serde value");
    println!("Retreived stream configuration");
    let unwrapped_stream_config = stream_config.expect("failed to convert from serde");
    println!("Decoded stream configuration");
    let file_path: String = String::from("output/video_") + &Uuid::new_v4().simple().to_string() + ".mp4";
    let mut format_context: OutputContext = FormatContext::new_output(CString::new(file_path.as_str()).unwrap());
    println!("Created output context");
    let encoding_context = try!(vid_processing::create_encoding_context(CodecId::from(AV_CODEC_ID_H264), 480, 640, Rational::new(1, 30), 12, 0));
    let pkt_stream = format_context.create_stream(encoding_context);

    let stream_index = pkt_stream.index;
    let stream_timebase = Rational::from(pkt_stream.time_base);
                    
    println!("Created output video stream");
    try!(open_video_file(file_path.as_ref(), &mut format_context));
    println!("Opened video file");
    try!(write_video_header(&mut format_context));
    println!("Wrote video header");

    while !completed_stream {
        let results = read_channel.read_next_message();

        match results {
            None => {
                println!("Read {} messages from stream, now reached EOS.", frames_read);
                completed_stream = true;
                try!(write_null_video_frame(&mut format_context));
                try!(write_video_trailer(&mut format_context));
                println!("Wrote video trailer and null video frame");
            }
            Some(v) => {
                if v == b"===ENDTRANSMISSION===" {
                    println!("received end of transmission");
                    completed_stream = true;
                    break;
                }
                frames_read = frames_read + 1;
                let data_packet_attempt = serde_cbor::from_slice::<DataPacket>(v.as_slice());
                match data_packet_attempt {
                    Ok(data_packet) => {
                        let mut packet = Packet::from(data_packet);
                        packet.dts = packet.pts;

                        println!("Recieved packet from client with pts {}", packet.pts);

                        let _ = try!(write_video_frame(&mut format_context, stream_index, packet));
                    },
                    Err(e) => { 
                        println!("Error found: {}", e);
                    },
                }
            }
        }
    }
    println!("Current read ended, {} frames read.", frames_read);
    try!(write_null_video_frame(&mut format_context));
    try!(write_video_trailer(&mut format_context));
    println!("Wrote video trailer and null video frame");
    Ok(frames_read)
}

#[derive(Debug)]
enum TranslatedRecordingInstructions {
    Start,
    Stop,
}

struct ServerToClientThreadHandler {
    rec_vid_thread: Cell<JoinHandle<()>>,
    currently_recv: bool,
    instr_tun: Sender<TranslatedRecordingInstructions>,
}

impl ServerToClientThreadHandler {
    fn new(mut read_channel: DualMessenger<TcpStream>) -> ServerToClientThreadHandler {
        let (send, recv) = channel();
        let rec_vid_thread = thread::spawn(move || {
            receive_video(read_channel, recv);
        });
        ServerToClientThreadHandler {
            rec_vid_thread: Cell::new(rec_vid_thread),
            currently_recv: false,
            instr_tun: send,
        }
    }

    fn start(&mut self) {
        self.instr_tun.send(TranslatedRecordingInstructions::Start);
    }

    fn stop(&mut self, mut read_channel: DualMessenger<TcpStream>) {
        let (instr_tx, instr_rx) = channel();
        let rec_vid_thread = thread::spawn(move || {
            receive_video(read_channel, instr_rx);
        });
        self.instr_tun = instr_tx;
        self.rec_vid_thread.replace(rec_vid_thread).join();

    }
}