use ffmpeg_sys::*;
use uuid::Uuid;

use std::net::{SocketAddr, TcpStream, TcpListener};
use client::errors::ClientError;
use std::io::{Write};
use std::fs::File;

use std::slice::from_raw_parts;
use std::ptr;
use std::ffi::CString;
use std::mem;

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

struct CodecStorage {
    encoding_context: *mut AVCodecContext,
    decoding_context: *mut AVCodecContext,
    jpeg_context: *mut AVCodecContext,
}

impl CodecStorage {

    fn new(enc: *mut AVCodecContext, dec: *mut AVCodecContext, jpeg: *mut AVCodecContext) -> CodecStorage {
        CodecStorage {
            encoding_context: enc,
            decoding_context: dec,
            jpeg_context: jpeg,
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
    let jpeg_tuple = allocate_jpeg_codec();

    let codec_storage: CodecStorage = CodecStorage::new(encoding_tuple.1, decoding_tuple.1, jpeg_tuple.1);

    //SWS ALLOCATION
    let sws_context: *mut SwsContext = allocate_sws_context();

    //INPUT ALLOCATION
    let mut input_context_ptr = ptr::null_mut();
    let input_format_ptr = av_find_input_format(CString::new("v4l2").unwrap().as_ptr());
    let mut opts_ptr = ptr::null_mut();

    let ret = avformat_open_input(&mut input_context_ptr, CString::new("/dev/video0").unwrap().as_ptr(), input_format_ptr, &mut opts_ptr); 
    if ret < 0 {
        panic!("couldn't open input, {}", ret);
    }

    av_dump_format(input_context_ptr, 0, CString::new("/dev/video0").unwrap().as_ptr(), 0);

    for pts in 0..100 {
        let pkt = av_packet_alloc();
        av_read_frame(input_context_ptr, pkt);

        transcode_packet(&codec_storage, sws_context, &(*pkt), pts, stream);
    }

    encode_raw_frame(encoding_tuple.1, ptr::null_mut(), stream);

}

unsafe fn transcode_packet(context_storage: &CodecStorage, sws_context: *mut SwsContext, input_packet: &AVPacket, pts: i64, stream: &mut TcpStream) -> Result<(), ClientError> {
    let raw_frame: *mut AVFrame = decode_raw_packet(context_storage.decoding_context, input_packet);

    let scaled_frame = change_pixel_format(raw_frame, sws_context, 32, pts);

    let file = try!(File::create(String::from("picture_") + Uuid::new_v4().to_string().as_ref() + String::from(".jpeg").as_ref()));
    encode_jpeg_frame(context_storage.jpeg_context, scaled_frame, file);

    encode_raw_frame(context_storage.encoding_context, scaled_frame, stream);

    Ok(())
}

unsafe fn change_pixel_format(old_frame: *mut AVFrame, sws_context: *mut SwsContext, align: i32, pts: i64) -> *mut AVFrame {
    let scaled_frame = av_frame_alloc();
    (*scaled_frame).width = (*old_frame).width;
    (*scaled_frame).height = (*old_frame).height;
    (*scaled_frame).format = 0;
    (*scaled_frame).pts = pts;

    let scaled_frame_data_ptr: *mut *mut u8 = (*scaled_frame).data.as_mut_ptr();
    let scaled_frame_linesize_ptr: *mut i32 = (*scaled_frame).linesize.as_mut_ptr();

    av_image_alloc(scaled_frame_data_ptr, scaled_frame_linesize_ptr, (*old_frame).width, (*old_frame).height, AV_PIX_FMT_YUV420P, align);

    let raw_frame_data_ptr: *const *const u8 = (*old_frame).data.as_ptr() as *const *const u8;
    let raw_frame_linesize_ptr: *mut i32 = (*old_frame).linesize.as_mut_ptr();

    let new_height = sws_scale(sws_context, raw_frame_data_ptr, raw_frame_linesize_ptr, 0, 480, scaled_frame_data_ptr, scaled_frame_linesize_ptr);

    scaled_frame
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
    let ret = avcodec_send_frame(codec, frame);

    if ret < 0 {
        panic!("issue sending the frame to the encoder: {}", ret);
    }

    while ret >= 0 {
        let packet = av_packet_alloc();
        let ret = avcodec_receive_packet(codec, packet);

        if ret == -11 {
            return;
        } else if ret == AVERROR_EOF {
            println!("EOF");
            return;
        } else if ret < 0 {
            panic!("issue receiving the frame from the encoder: {}", ret);
        }

        stream.write(from_raw_parts((*packet).data, (*packet).size as usize));
        av_packet_unref(packet);

    }

}

unsafe fn encode_jpeg_frame(codec: *mut AVCodecContext, frame: *mut AVFrame, mut file: File) {
    let packet = av_packet_alloc();
    let ret = avcodec_send_frame(codec, frame);

    if ret < 0 {
        panic!("issue sending the frame to the encoder: {}", ret);
    }

    let ret = avcodec_receive_packet(codec, packet);

    if ret == -11 || ret == AVERROR_EOF {
        print!("e");
        return;
    } else if ret < 0 {
        panic!("issue receiving the frame from the encoder: {}", ret);
    }

    let _ = file.write(from_raw_parts((*packet).data, (*packet).size as usize));
    av_packet_unref(packet);
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
    decoding_context.pix_fmt = AV_PIX_FMT_YUYV422;

    if avcodec_open2(decoding_context_ptr, codec_ptr, ptr::null_mut()) < 0 {
        panic!("couldn't open decoding codec");
    }

    (codec_ptr, decoding_context_ptr)


}

unsafe fn allocate_jpeg_codec() -> (*mut AVCodec, *mut AVCodecContext) {

    let codec_ptr: *mut AVCodec = avcodec_find_encoder(AV_CODEC_ID_JPEG2000);
    let jpeg_context_ptr: *mut AVCodecContext = avcodec_alloc_context3(codec_ptr);

    let ref mut jpeg_context: AVCodecContext = *jpeg_context_ptr;

    jpeg_context.height = 480;
    jpeg_context.width = 640;

    jpeg_context.time_base = av_make_q(1, 25);

    jpeg_context.pix_fmt = AV_PIX_FMT_YUV420P;

    if avcodec_open2(jpeg_context_ptr, codec_ptr, ptr::null_mut()) < 0 {
        panic!("couldn't open decoding codec");
    }

    (codec_ptr, jpeg_context_ptr)


}

unsafe fn allocate_sws_context() -> (*mut SwsContext) {
    let cached = sws_getCachedContext(ptr::null_mut(), 640, 480, AV_PIX_FMT_YUYV422, 640, 480, AV_PIX_FMT_YUV420P, SWS_BICUBIC, ptr::null_mut(), ptr::null_mut(), ptr::null());

    if cached.is_null() {
        panic!("failed to alloc cached context");
    }

    cached
}