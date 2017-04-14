use ffmpeg_sys::*;
use libc;

use std::net::{SocketAddr, TcpStream, TcpListener};
use client::errors::ClientError;
use std::io::{Write};

use std::slice::from_raw_parts;
use std::ptr;
use std::ffi::CString;

#[derive(Debug)]
pub struct Client {
    server_ip: SocketAddr,
    listener: TcpListener,
}

impl Client {
    pub fn new(sock: (SocketAddr, SocketAddr)) -> Result<Client, ClientError> {
        let mut stream = try!(TcpStream::connect(sock.0));

        let output_string = sock.1.port().to_string();
        let mut out_vec = output_string.into_bytes();
        pad_output_vec(&mut out_vec, 8);

        let _ = stream.write(out_vec.as_slice());
        let _ = stream.write(b"INTC_01");

        let listener = try!(TcpListener::bind(sock.1));

        Ok(Client { server_ip: sock.0, listener: listener })
    }

    pub fn handle(&self) -> Result<(), ClientError> {
        for stream in self.listener.incoming() {
            let mut stream = try!(stream);
            println!("Connection recieved: {:?}", stream);
            unsafe {
                send_video(&mut stream);
            }
        }

        Ok(())
    }
}

fn pad_output_vec(vec: &mut Vec<u8>, pad_by: usize) {
    for _ in 0..pad_by {
        if vec.len() >= pad_by {
            break;
        } else {
            vec.push(0);
        }
    }
}

unsafe fn send_video(stream: &mut TcpStream) {
    av_register_all();
    avdevice_register_all();
    
    let mut input_context_ptr = ptr::null_mut();
    let input_format_ptr = av_find_input_format(CString::new("v4l2").unwrap().as_ptr());
    let mut opts_ptr = ptr::null_mut();

    let ret = avformat_open_input(&mut input_context_ptr, CString::new("/dev/video0").unwrap().as_ptr(), input_format_ptr, &mut opts_ptr); 
    if ret < 0 {
        panic!("couldn't open input, {}", ret);
    }

    av_dump_format(input_context_ptr, 0, CString::new("/dev/video0").unwrap().as_ptr(), 0);

    for _ in 0..25 {
        let mut pkt = av_packet_alloc();
        av_read_frame(input_context_ptr, pkt);

        stream.write(from_raw_parts((*pkt).data, (*pkt).size as usize));

        av_packet_free(&mut pkt);
    }

}