use std::net::{SocketAddr, TcpStream, TcpListener};
use client::errors::ClientError;
use std::io::{Write};

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

    pub fn handle(&self) {
        for stream in self.listener.incoming() {
            println!("Connection recieved: {:?}", stream)
        }
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
