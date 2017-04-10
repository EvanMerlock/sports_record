use std::net::{TcpListener, TcpStream, SocketAddr};
use std::thread;
use std::thread::JoinHandle;
use std::result::Result;
use std::path::{Path, PathBuf};
use std::fs::File;
use std::sync::{Arc, RwLock};
use std::sync::mpsc::{Sender, Receiver, channel};
use std::io::Read;

use server::errors::{ ServerError };

pub struct ClientStream {
    socket_channel: Receiver<ClientIPInformation>,

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
    Add(SocketAddr),
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
        self.thread_channel.send(RecordingInstructions::Cleanup);
    }
}

impl ClientStream {

    pub fn new() -> Result<ClientStream, ServerError> {
        let (socket_channel_send, socket_chanel_recv) = channel();
        let (client_ip_send, client_ip_recv) = channel();
        let (instruct_send, instruct_recv) = channel();

        let client_handler_channel = thread::spawn(move || {
            main_client_handler(client_ip_recv, instruct_recv);
        });

        let stream = ClientStream {
            socket_channel: socket_chanel_recv,

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
                    item.thread_channel.send(current_instruction);
                }

                end_exec = true;
            },
            _ => {
                handle_client_changes(&mut thread_list, &ip_channel);

                for item in &thread_list {
                    item.thread_channel.send(current_instruction);
                }
            }
        }
    }

}

fn handle_client_changes(thread_list: &mut Vec<ClientThreadInformation>, ip_channel: &Receiver<ClientIPInformation>) {
    for ip_info in ip_channel.try_iter() {
        match ip_info {
            ClientIPInformation::Add(i) => {
                let (send, recv) = channel();
                let temp_thread_handle = thread::spawn(move || {
                    let _ = individual_client_handler(i.clone(), recv);
                });
                thread_list.push(ClientThreadInformation::new(i.clone(), temp_thread_handle, send));
            }, 
            ClientIPInformation::Remove(i) => {
                thread_list.retain(|x| !(x.socket_addr == i));
            },
        }
    }
}

fn individual_client_handler(address: SocketAddr, recv: Receiver<RecordingInstructions>) -> Result<(), ServerError> {

    let curr_instruction = recv.recv().unwrap();
    let mut currently_cleaning = false;
    let mut currently_executing_instruction = false;
    
    while !currently_cleaning {
        while !currently_executing_instruction {
            match curr_instruction {
                RecordingInstructions::StartRecording(i) => {
                    currently_executing_instruction = true;
                    let mut curr_tcp_stream = try!(TcpStream::connect(address));

                    let mut completed_stream = false;

                    while !completed_stream {
                        let mut buffer = [0; 8];
                        let results = curr_tcp_stream.read(&mut buffer);

                        match results {
                            Ok(i) if i == 0 => {
                                println!("EOS in individual_client_handler");
                                completed_stream = true;
                            }
                            Ok(..) => {
                                println!("Received Data: {:?}", buffer);
                            }
                            Err(e) => {
                                println!("Error: {:?}", e);
                            }
                        }
                    }
                },
                RecordingInstructions::StopRecording => {
                    println!("StopRecording not currently supported!");
                    currently_executing_instruction = false;
                }
                RecordingInstructions::Cleanup => {
                    currently_cleaning = true;
                    currently_executing_instruction = false;
                }
            }
        }
    }

    Ok(())
}