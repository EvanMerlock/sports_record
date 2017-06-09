use uuid::Uuid;

use std::net::{TcpStream, SocketAddr, Shutdown};
use std::thread;
use std::thread::JoinHandle;
use std::result::Result;
use std::sync::mpsc::{Sender, Receiver, channel};
use std::io::{Read, Write};
use std::fs::File;

use std::slice::from_raw_parts;
use std::ffi::CString;

use server::{ServerError, ServerErrorKind};
use messenger_plus::stream::{DualMessenger};
use config::stream_config::StreamConfiguration;
use unsafe_code::vid_processing;
use unsafe_code::output::{FormatContext, open_video_file, write_video_frame, write_video_header, write_video_trailer, write_null_video_frame};
use unsafe_code::{Rational};
use unsafe_code::packet::{Packet, DataPacket};
use unsafe_code::img_processing;
use config::codec_parameters::put_raw_codecpars_into_stream;
use serde_json;
use bincode;

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

#[derive(Debug, Copy, Clone)]
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
                handle_client_changes(&mut thread_list, &ip_channel);

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
                        thread_info.thread_channel.send(RecordingInstructions::Cleanup);
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

    let mut dual_channel: DualMessenger<TcpStream> = DualMessenger::new(String::from("--"), String::from("boundary"), String::from("endboundary"), &mut stream);

    while !currently_cleaning {
        loop {
            let curr_instruction = recv.recv().unwrap();
            match curr_instruction {
                RecordingInstructions::StartRecording(i) => {
                    println!("writing to channel");
                    let _ = dual_channel.write(b"START");

                    let mut completed_stream = false;
                    let mut frames_read = 0;

                    println!("reading next message");
                    let results = dual_channel.read_next_message();
                    let stream_config = results.map(|x| {
                        serde_json::from_slice::<StreamConfiguration>(x.as_ref())
                    }).expect("failure to collect serde value");
                    println!("created serde value");
                    let unwrapped_stream_config = stream_config.expect("failed to convert from serde");
                    println!("unwrapped serde value");
                    let mut format_context: FormatContext = FormatContext::new(CString::new("video.mp4").unwrap());
                    println!("generated format context");
                    let mut encoding_context = try!(vid_processing::create_encoding_context(AV_CODEC_ID_H264, 480, 640, Rational::new(1, 30), 12, 0));
                    let mut pkt_stream = format_context.create_stream(encoding_context);
                    
                    println!("created stream");
                    try!(open_video_file("video.mp4", &mut format_context));
                    println!("opened vid file");
                    try!(write_video_header(&mut format_context));
                    println!("wrote video header");
                    // let mut decoding_context = try!(vid_processing::create_decoding_context_from_stream_configuration(unwrapped_stream_config));
                    // let mut jpeg_context = try!(img_processing::create_jpeg_context(480, 640, Rational::new(1, 30)));

                    while !completed_stream {
                        let results = dual_channel.read_next_message();

                        match results {
                            None => {
                                println!("Read {} messages from stream, now reached EOS.", frames_read);
                                completed_stream = true;
                                try!(write_null_video_frame(&mut format_context));
                                try!(write_video_trailer(&mut format_context));
                                println!("Wrote video trailer and null video frame");
                            }
                            Some(v) => {
                                frames_read = frames_read + 1;

                                let mut data_packet: DataPacket = try!(bincode::deserialize(v.as_slice()));
                                let mut packet = Packet::from(data_packet);
                                packet.dts = packet.pts;

                                println!("Recieved packet from client");

                                let raw_frame_chance = try!(write_video_frame(&mut format_context, &pkt_stream, packet));
                                //let file = try!(File::create(String::from("output/picture_") + Uuid::new_v4().to_string().as_ref() + String::from(".jpeg").as_ref()));
                                //try!(img_processing::write_frame_to_jpeg(&mut jpeg_context, raw_frame, file));
                            }
                        }
                    }
                },
                RecordingInstructions::StopRecording => {
                    let _ = dual_channel.write(b"STOP");
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
    dual_channel.release().shutdown(Shutdown::Both);
    Ok(())
}