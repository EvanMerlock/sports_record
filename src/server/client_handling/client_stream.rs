use std::net::{TcpStream, SocketAddr, Shutdown};
use std::thread;
use std::thread::JoinHandle;
use std::result::Result;
use std::sync::mpsc::{Sender, Receiver, channel};
use std::io::{Read, Write};
use std::fs::File;

use server::ServerError;
use messenger_plus::stream::{DualMessenger};

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
                    let _ = individual_client_handler(i, recv);
                });
                thread_list.push(ClientThreadInformation::new(socket_addr, temp_thread_handle, send));
            }, 
            ClientIPInformation::Remove(i) => {
                //thread_list.retain(|x| !(x.socket_addr == i));
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
                    let _ = dual_channel.write(b"START");
                    println!("sent start code");
                    let mut curr_file = try!(File::create("video.mp4"));

                    let mut completed_stream = false;

                    while !completed_stream {
                        println!("reading from dual channel");
                        let results = dual_channel.read_next_message();

                        match results {
                            None => {
                                println!("Client ended stream");
                                completed_stream = true;
                            }
                            Some(v) => {
                                let _ = curr_file.write(v.as_slice());
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