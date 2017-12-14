use std::net::{TcpStream, SocketAddr, Shutdown};
use std::thread;
use std::thread::JoinHandle;
use std::result::Result;
use std::sync::{Arc, Weak, Mutex, MutexGuard};
use std::sync::mpsc::{Sender, Receiver, channel, TryRecvError};
use std::io::Write;
use std::cell::Cell;
use std::default::Default;

use std::ffi::CString;

use server::{ServerError, sql};
use ffmpeg_common::unsafe_code::StreamConfiguration;

use ffmpeg_common::unsafe_code::format::{FormatContext, OutputContext};
use ffmpeg_common::unsafe_code::{EncodingCodecContext, Rational, CodecId, Packet, UnsafeError, UnsafeErrorKind};
use ffmpeg_common::networking::{NetworkPacket, NetworkConfiguration};

use uuid::Uuid;
use serde_json;
use messenger_plus::stream::{DualMessenger};
use messenger_plus::stream;

use iron::typemap;

use ffmpeg_sys::*;

#[derive(Clone)]
pub struct ClientStream {
    current_clients: Arc<Mutex<Vec<ClientThreadInformation>>>,
    db_access: sql::DatabaseRef,
    out_dir: String
}

#[derive(Clone)]
pub struct WeakClientStream {
    current_clients: Weak<Mutex<Vec<ClientThreadInformation>>>,
}

pub struct ClientThreadInformation {
    socket_addr: SocketAddr,
    pub ws_url: SocketAddr,
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
    StartRecording,
    StopRecording,
    Cleanup,
}

impl ClientThreadInformation {
    pub fn new(sock: SocketAddr, tcp_stream: TcpStream, db_ref: sql::DatabaseRef, out_dir: String) -> Result<ClientThreadInformation, UnsafeError> {
        let stream = tcp_stream.try_clone()?;
        println!("Attempting to retrieve stream configuration from client {}", stream.peer_addr()?);
        let mut read_channel: DualMessenger<TcpStream> = DualMessenger::new(String::from("--"), String::from("boundary"), String::from("endboundary"), stream);

        let results = read_channel.read_next_message().map_err(|x| UnsafeError::from(x))?;
        let stream_config = serde_json::from_slice::<NetworkPacket>(results.as_ref()).map_err(|x| UnsafeError::from(x))?;
        let unwrapped_config = match stream_config {
            NetworkPacket::JSONPayload(e) => e,
            _ => return Err(UnsafeError::new(UnsafeErrorKind::OpenInput(1000))),
        };
        println!("Retreived stream configuration from client {}", tcp_stream.peer_addr()?);
        println!("{:?}", unwrapped_config);

        let (send, recv) = channel();
        let ws_sock = unwrapped_config.websocket_address.clone(); 
        let thread_handle = thread::spawn(move || {
            let val = client_write_handler(tcp_stream, recv, db_ref, out_dir, unwrapped_config);
            println!("{:?}", val);
        });
        Ok(ClientThreadInformation { socket_addr: sock, thread_handle: thread_handle, thread_channel: send, ws_url: ws_sock })
    }
}

impl Drop for ClientThreadInformation {
    fn drop(&mut self) {
        let _ = self.thread_channel.send(RecordingInstructions::Cleanup);
    }
}

impl ClientStream {

    pub fn new(db_ref: sql::DatabaseRef, out_dir: String) -> Result<ClientStream, ServerError> {
        let stream = ClientStream {
            current_clients: Arc::new(Mutex::new(vec![])),
            db_access: db_ref,
            out_dir: out_dir,
        };
        Ok(stream)
    }

    pub fn add_client(&mut self, info: TcpStream) -> Result<(), ServerError> {
        let socket_addr = try!(info.peer_addr());
        let mut lock = self.current_clients.lock().unwrap();
        lock.push(ClientThreadInformation::new(socket_addr, info, self.db_access.clone(), self.out_dir.clone())?);
        Ok(())
    }

    pub fn remove_client(&mut self, info: SocketAddr) {
        let mut new_thread_list: Vec<ClientThreadInformation> = Vec::new();

        let mut lock = self.current_clients.lock().unwrap();
        for thread_info in lock.drain(0..) {
            if !(thread_info.socket_addr == info) {
                new_thread_list.push(thread_info);
            } else {
                let _ = thread_info.thread_channel.send(RecordingInstructions::Cleanup);
            }
        }
        lock.append(&mut new_thread_list);
    }

    pub fn get_client_view(&self) -> MutexGuard<Vec<ClientThreadInformation>> {
        self.current_clients.lock().expect("mutex poisoned")
    }

    pub fn start_recording(&self) {
        let _ = self.db_access.start_play();
        self.send_command(RecordingInstructions::StartRecording);
    }

    pub fn stop_recording(&self) {
        self.db_access.end_play();
        self.send_command(RecordingInstructions::StopRecording);
    }

    pub fn clean_up(&mut self) {
        self.send_command(RecordingInstructions::Cleanup);
        let mut lock = self.current_clients.lock().unwrap();
        lock.clear();
    }

    fn send_command(&self, current_command: RecordingInstructions) {
        let lock = self.current_clients.lock().unwrap();
        for item in &*lock {
            let _ = item.thread_channel.send(current_command);
        }
    }

    pub fn get_weak(&self) -> WeakClientStream {
        WeakClientStream {
            current_clients: Arc::downgrade(&self.current_clients)
        }
    }

}

impl WeakClientStream {
    pub fn get_client_view(&self) -> Option<Arc<Mutex<Vec<ClientThreadInformation>>>> {
        self.current_clients.upgrade()
    }
}

impl typemap::Key for WeakClientStream {
    type Value = WeakClientStream;
}

fn client_write_handler(stream: TcpStream, recv: Receiver<RecordingInstructions>, db_ref: sql::DatabaseRef, out_dir: String, cfg: NetworkConfiguration) -> Result<(), ServerError> {

    let mut currently_cleaning = false;

    let write_stream = try!(stream.try_clone());
    let read_stream = try!(stream.try_clone());
    
    let mut read_channel: DualMessenger<TcpStream> = DualMessenger::new(String::from("--"), String::from("boundary"), String::from("endboundary"), read_stream);
    let mut write_channel: DualMessenger<TcpStream> = DualMessenger::new(String::from("--"), String::from("boundary"), String::from("endboundary"), write_stream);

    let mut stcth = LoopingThreadHandler::new(cfg.stream_configuration, read_channel, db_ref, out_dir);

    while !currently_cleaning {
        loop {
            let curr_instruction = recv.recv().unwrap();
            match curr_instruction {
                RecordingInstructions::StartRecording => {
                    let _ = write_channel.write(b"START");
                    stcth.start();
                },
                RecordingInstructions::StopRecording => {
                    let _ = write_channel.write(b"STOP");
                    stcth.stop();
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

fn looping_recv_video(conf: StreamConfiguration, mut read_channel: DualMessenger<TcpStream>, instr_recv: Receiver<TranslatedRecordingInstructions>, db_ref: sql::DatabaseRef, out_dir: String) -> Result<(), ServerError> {

    let mut currently_recv = false;
    let mut on_ending_payload = false;
    let mut frames_read = 0;
    let mut current_output_context = Cell::new(Option::None);
    let stream_timebase = Cell::new(Rational::default());
    let stream_index = Cell::new(0);


    let encoding_context = EncodingCodecContext::create_encoding_context(CodecId::from(AVCodecID::AV_CODEC_ID_H264), conf.height, conf.width, conf.time_base, conf.gop_size, conf.max_b_frames)?;
    println!("Created encoding context");

    // internal loop
    loop {
        match instr_recv.try_recv() {
            Ok(ref m) if m == &TranslatedRecordingInstructions::Start => {
                currently_recv = true;
                let uuid: String = Uuid::new_v4().simple().to_string();
                (&db_ref).insert_clip(&uuid)?;
                let file_path: String = out_dir.clone() + "/video_" + &uuid + ".mp4";
                let mut format_context: OutputContext = FormatContext::new_output(CString::new(file_path.as_str()).unwrap());
                println!("Created output context");
                let pkt_stream = format_context.create_stream(&encoding_context);
                println!("Created output video stream");
                try!(format_context.open_video_file(file_path.as_ref()));
                println!("Opened video file: {}", file_path.as_str());
                try!(format_context.write_video_header());
                println!("Wrote video header");
                current_output_context.replace(Option::Some(format_context));
                stream_index.replace(pkt_stream.index);
                stream_timebase.replace(Rational::from(pkt_stream.time_base));
                frames_read = 0;
            },
            Err(ref e) if (e != &TryRecvError::Empty) => {
                eprintln!("An unexpected error occured within the server. Please restart the server and the client {:?}", e);
                break;
            }
            _ => {},
        }

        if currently_recv {
            let res = read_channel.read_next_message();
            match res {
                Err(ref e) if e == &stream::Error::from(stream::ErrorKind::BufferEmpty) => {
                    println!("Read {} messages from stream, now reached EOS.", frames_read);
                    currently_recv = false;
                    on_ending_payload = true;
                },
                Ok(v) => {
                    frames_read = frames_read + 1;
                    let data_packet_attempt = serde_json::from_slice::<NetworkPacket>(v.as_slice()).map_err(|x| UnsafeError::from(x));
                    match data_packet_attempt {
                        Ok(network_packet) => {
                            match network_packet {
                                NetworkPacket::PacketStream(pkts) => {
                                    for mut pkt in pkts.into_iter().map(|x| Packet::from(x)) {
                                        println!("Recieved packet from client with pts {}", pkt.pts);
                                        pkt.rescale_to(Rational::new(1,30), stream_timebase.get());
                                        let format_context = current_output_context.get_mut().as_mut().expect("desync");
                                        let _ = format_context.write_video_frame(stream_index.get(), pkt)?;
                                    }
                                },
                                NetworkPacket::PayloadEnd => {
                                    println!("Received EOP Indicator");
                                    on_ending_payload = true;
                                    currently_recv = false;
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

        if on_ending_payload {
            let mut temp_context = current_output_context.replace(Option::None);
            let format_context = temp_context.as_mut().expect("desync");
            println!("Current read ended, {} frames read.", frames_read);
            try!(format_context.write_null_video_frame());
            try!(format_context.write_video_trailer());
            println!("Wrote video trailer and null video frame");
            on_ending_payload = false;
        }
    }
    Ok(())
}

#[derive(Debug, PartialEq)]
enum TranslatedRecordingInstructions {
    Start,
    Stop,
}

struct LoopingThreadHandler {
    rec_vid_thread: JoinHandle<Result<(), ServerError>>,
    instr_tun: Sender<TranslatedRecordingInstructions>,
    conf: StreamConfiguration,
}

impl LoopingThreadHandler {
    fn new(conf: StreamConfiguration, read_channel: DualMessenger<TcpStream>, db_ref: sql::DatabaseRef, out_dir: String) -> LoopingThreadHandler {
        let (send, recv) = channel();
        let rec_vid_thread = thread::spawn(move || {
            let x = looping_recv_video(conf, read_channel, recv, db_ref, out_dir);
            println!("{:?}", x);
            x
        });
        LoopingThreadHandler {
            rec_vid_thread: rec_vid_thread,
            instr_tun: send,
            conf: conf,
        }
    }

    fn start(&mut self) {
        let _ = self.instr_tun.send(TranslatedRecordingInstructions::Start);
    }

    fn stop(&mut self) {
        let _ = self.instr_tun.send(TranslatedRecordingInstructions::Stop);
    }
}