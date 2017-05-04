use unsafe_code::{UnsafeError, UnsafeErrorKind};
use messenger_plus::stream::DualMessenger;

use std::net::TcpStream;
use std::io::Write;

use std::ptr;
use std::ffi::CString;
use std::slice::from_raw_parts;

use ffmpeg_sys::*;

unsafe fn allocate_encoding_codec(codec_type: AVCodecID, height: i32, width: i32, time_base: AVRational, framerate: AVRational) -> Result<(*mut AVCodec, *mut AVCodecContext), UnsafeError> {

    let codec_ptr: *mut AVCodec = avcodec_find_encoder(codec_type);
    let encoding_context_ptr: *mut AVCodecContext = avcodec_alloc_context3(codec_ptr);

    let ref mut encoding_context: AVCodecContext = *encoding_context_ptr;

    encoding_context.bit_rate = 4000000;
    encoding_context.height = height;
    encoding_context.width = width;

    encoding_context.time_base = time_base;
    encoding_context.framerate = framerate;

    encoding_context.gop_size = 10;
    encoding_context.max_b_frames = 1;
    encoding_context.pix_fmt = AV_PIX_FMT_YUV420P;

    if codec_type == AV_CODEC_ID_H264 {
        av_opt_set(encoding_context.priv_data, CString::new("preset").unwrap().as_ptr(), CString::new("slow").unwrap().as_ptr(), 0);
    }

    let ret = avcodec_open2(encoding_context_ptr, codec_ptr, ptr::null_mut());
    if ret < 0 {
        return Err(UnsafeError::new(UnsafeErrorKind::OpenEncoder(ret)));
    }

    Ok((codec_ptr, encoding_context_ptr))

}

pub fn create_encoding_context<'a>(codec_type: AVCodecID, height: i32, width: i32, time_base: AVRational, framerate: AVRational) -> Result<&'a mut AVCodecContext, UnsafeError> {
    unsafe {
        match allocate_encoding_codec(codec_type, height, width, time_base, framerate) {
            Ok((_, context)) => Ok(&mut *context),
            Err(e) => Err(e),
        }
    }
}

unsafe fn encode_raw_frame(codec: &mut AVCodecContext, frame: *mut AVFrame, stream: &mut DualMessenger<TcpStream>) -> Result<(), UnsafeError> {    
    let ret = avcodec_send_frame(codec, frame);

    if ret < 0 {
        return Err(UnsafeError::new(UnsafeErrorKind::SendFrame(ret)));
    }

    while ret >= 0 {
        let packet = av_packet_alloc();
        let ret = avcodec_receive_packet(codec, packet);

        if ret == -11 || ret == AVERROR_EOF {
            return Ok(());
        } else if ret < 0 {
            return Err(UnsafeError::new(UnsafeErrorKind::ReceivePacket(ret)));
        }

        let res = stream.write(from_raw_parts((*packet).data, (*packet).size as usize));
        println!("res: {:?}", res);
        av_packet_unref(packet);

    }

    Ok(())

}

pub fn encode_frame(context: &mut AVCodecContext, frame: &mut AVFrame, stream: &mut DualMessenger<TcpStream>) -> Result<(), UnsafeError> {
    unsafe {
        encode_raw_frame(context, frame, stream)
    }
}

pub fn encode_null_frame(context: &mut AVCodecContext, stream: &mut DualMessenger<TcpStream>) -> Result<(), UnsafeError> {
    unsafe {
        encode_raw_frame(context, ptr::null_mut(), stream)
    }
}