use std::net::{TcpStream};
use std::io::Write;

pub fn video_receiver(mut stream: TcpStream) {
    let _ = stream.write(b"START_VIDEO");
}