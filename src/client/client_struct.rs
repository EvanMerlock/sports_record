use ffmpeg_sys::*;

use std::net::{SocketAddr, TcpStream, TcpListener};
use client::errors::ClientError;
use std::io::{Write};
use std::thread;
use std::time::Duration;

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
    avcodec_register_all();

    //CODEC ALLOCATION
    let encoding_tuple = allocate_encoding_codec();
    let decoding_tuple = allocate_decoding_codec();

    //INPUT ALLOCATION
    let mut input_context_ptr = ptr::null_mut();
    let input_format_ptr = av_find_input_format(CString::new("v4l2").unwrap().as_ptr());
    let mut opts_ptr = ptr::null_mut();

    let ret = avformat_open_input(&mut input_context_ptr, CString::new("/dev/video0").unwrap().as_ptr(), input_format_ptr, &mut opts_ptr); 
    if ret < 0 {
        panic!("couldn't open input, {}", ret);
    }

    av_dump_format(input_context_ptr, 0, CString::new("/dev/video0").unwrap().as_ptr(), 0);

    for _ in 0..25 {
        let pkt = av_packet_alloc();
        av_read_frame(input_context_ptr, pkt);

        transcode_packet(decoding_tuple.1, encoding_tuple.1, &(*pkt), stream);
    }

    encode_raw_frame(encoding_tuple.1, ptr::null_mut(), stream);

}

unsafe fn transcode_packet(decoding_context: *mut AVCodecContext, encoding_context: *mut AVCodecContext, input_packet: &AVPacket, stream: &mut TcpStream) {
    let raw_frame = decode_raw_packet(decoding_context, input_packet);
    encode_raw_frame(encoding_context, raw_frame, stream)
}

unsafe fn decode_raw_packet(codec: *mut AVCodecContext, packet: &AVPacket) -> *mut AVFrame {
    let frame = av_frame_alloc();

    let ret = avcodec_send_packet(codec, packet);

    if ret < 0 {
        panic!("issue sending the packet to the decoder: {}", ret);
    }

    let ret = avcodec_receive_frame(codec, frame);

    if ret < 0 {
        panic!("issue receiving the frame from the decoder: {}", ret);
    }

    frame
}

unsafe fn encode_raw_frame(codec: *mut AVCodecContext, frame: *mut AVFrame, stream: &mut TcpStream) {    
    let packet = av_packet_alloc();
    let ret = avcodec_send_frame(codec, frame);

    if ret < 0 {
        panic!("issue sending the frame to the encoder: {}", ret);
    }

    while ret >= 0 {
        let ret = avcodec_receive_packet(codec, packet);

        if ret == -11 || ret == AVERROR_EOF {
            return;
        } else if ret < 0 {
            panic!("issue receiving the frame from the encoder: {}", ret);
        }

        stream.write(from_raw_parts((*packet).data, (*packet).size as usize));
        av_packet_unref(packet);

    }

}

unsafe fn allocate_encoding_codec() -> (*mut AVCodec, *mut AVCodecContext) {

    let codec_ptr: *mut AVCodec = avcodec_find_encoder(AV_CODEC_ID_H264);
    let encoding_context_ptr: *mut AVCodecContext = avcodec_alloc_context3(codec_ptr);

    let ref mut encoding_context: AVCodecContext = *encoding_context_ptr;

    encoding_context.bit_rate = 4000000;
    encoding_context.height = 480;
    encoding_context.width = 640;

    encoding_context.time_base = av_make_q(1, 25);
    encoding_context.framerate = av_make_q(25, 1);

    encoding_context.gop_size = 10;
    encoding_context.max_b_frames = 1;
    encoding_context.pix_fmt = AV_PIX_FMT_YUV420P;

    av_opt_set(encoding_context.priv_data, CString::new("preset").unwrap().as_ptr(), CString::new("slow").unwrap().as_ptr(), 0);

    if avcodec_open2(encoding_context_ptr, codec_ptr, ptr::null_mut()) < 0 {
        panic!("couldn't open encoding codec!");
    }

    (codec_ptr, encoding_context_ptr)

}

unsafe fn allocate_decoding_codec() -> (*mut AVCodec, *mut AVCodecContext) {

    let codec_ptr: *mut AVCodec = avcodec_find_decoder(AV_CODEC_ID_RAWVIDEO);
    let decoding_context_ptr: *mut AVCodecContext = avcodec_alloc_context3(codec_ptr);

    let ref mut decoding_context: AVCodecContext = *decoding_context_ptr;

    decoding_context.height = 480;
    decoding_context.width = 640;

    decoding_context.time_base = av_make_q(1, 25);
    decoding_context.framerate = av_make_q(25, 1);

    decoding_context.gop_size = 10;
    decoding_context.max_b_frames = 1;
    decoding_context.pix_fmt = AV_PIX_FMT_YUV422P;

    if avcodec_open2(decoding_context_ptr, codec_ptr, ptr::null_mut()) < 0 {
        panic!("couldn't open decoding codec");
    }

    (codec_ptr, decoding_context_ptr)


}