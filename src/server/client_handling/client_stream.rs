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
    pub fn new(sock: SocketAddr, mut tcp_stream: TcpStream) -> ClientThreadInformation {
        let (send, recv) = channel();
        let thread_handle = thread::spawn(move || {
            let val = individual_client_handler(tcp_stream, recv);
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
                    println!("EOT");
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