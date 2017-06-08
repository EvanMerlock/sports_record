use unsafe_code::{UnsafeError, UnsafeErrorKind, CodecContext, Rational};
use config::stream_config::*;

use std::fs::File;
use std::io::Write;

use std::ptr;
use std::slice::from_raw_parts;
use std::ffi::CString;

use ffmpeg_sys::*;
use libc;

unsafe fn allocate_decoding_codec_from_stream_config(stream_config: StreamConfiguration) -> Result<(*mut AVCodec, *mut AVCodecContext), UnsafeError> {

    let codec_ptr: *mut AVCodec = avcodec_find_decoder(*stream_config.codec_id);
    let decoding_context_ptr: *mut AVCodecContext = avcodec_alloc_context3(codec_ptr);

    let ref mut decoding_context: AVCodecContext = *decoding_context_ptr;

    decoding_context.bit_rate = stream_config.bit_rate;
    decoding_context.height = stream_config.height;
    decoding_context.width = stream_config.width;

    decoding_context.time_base = stream_config.time_base.into();

    decoding_context.framerate = stream_config.frame_rate.into();

    // decoding_context.gop_size = stream_config.gop_size;
    // decoding_context.max_b_frames = stream_config.max_b_frames;

    decoding_context.gop_size = 0;
    decoding_context.max_b_frames = 0;

    decoding_context.pix_fmt = *stream_config.pix_fmt;

    if *stream_config.codec_id == AV_CODEC_ID_H264 {
        av_opt_set((decoding_context as *mut AVCodecContext) as *mut libc::c_void, CString::new("preset").unwrap().as_ptr(), CString::new("faster").unwrap().as_ptr(), 0);
        av_opt_set((decoding_context as *mut AVCodecContext) as *mut libc::c_void, CString::new("crf").unwrap().as_ptr(), CString::new("28").unwrap().as_ptr(), 0);
    }

    let ret = avcodec_open2(decoding_context_ptr, codec_ptr, ptr::null_mut());
    if ret < 0 {
        return Err(UnsafeError::new(UnsafeErrorKind::OpenDecoder(ret)));
    }

    Ok((codec_ptr, decoding_context_ptr))
}

pub fn create_decoding_context_from_stream_configuration(stream: StreamConfiguration) -> Result<CodecContext, UnsafeError> {
    unsafe {
        match allocate_decoding_codec_from_stream_config(stream) {
            Ok((_, context)) => Ok(CodecContext::from(context)),
            Err(e) => Err(e),
        }
    }
}

unsafe fn allocate_decoding_codec_from_av_stream(stream_config: &mut AVStream, timebase: Rational) -> Result<(*mut AVCodec, *mut AVCodecContext), UnsafeError> {
    let codec_ptr: *mut AVCodec = avcodec_find_decoder((*stream_config.codecpar).codec_id);
    let decoding_context_ptr: *mut AVCodecContext = avcodec_alloc_context3(codec_ptr);

    let ref mut decoding_context: AVCodecContext = *decoding_context_ptr;
    let ret = avcodec_parameters_to_context(decoding_context_ptr, stream_config.codecpar);
    if ret < 0 {
        return Err(UnsafeError::new(UnsafeErrorKind::OpenDecoder(ret)));
    }


    if decoding_context.codec_id == AV_CODEC_ID_H264 {
        av_opt_set((decoding_context as *mut AVCodecContext) as *mut libc::c_void, CString::new("preset").unwrap().as_ptr(), CString::new("ultrafast").unwrap().as_ptr(), 0);
        av_opt_set((decoding_context as *mut AVCodecContext) as *mut libc::c_void, CString::new("crf").unwrap().as_ptr(), CString::new("28").unwrap().as_ptr(), 0);
    }

    let ret = avcodec_open2(decoding_context_ptr, codec_ptr, ptr::null_mut());
    if ret < 0 {
        return Err(UnsafeError::new(UnsafeErrorKind::OpenDecoder(ret)));
    }

    Ok((codec_ptr, decoding_context_ptr))
}

pub fn create_decoding_context_from_av_stream(stream: &mut AVStream, timebase: Rational) -> Result<CodecContext, UnsafeError> {
    unsafe {
        match allocate_decoding_codec_from_av_stream(stream, timebase) {
            Ok((_, context)) => Ok(CodecContext::from(context)),
            Err(e) => Err(e),
        }
    }
}

unsafe fn decode_raw_packet<'a>(codec: &mut AVCodecContext, packet: &AVPacket) -> Result<&'a mut AVFrame, UnsafeError> {
    let frame = av_frame_alloc();

    let ret = avcodec_send_packet(codec, packet);

    if ret != 0 {
        return Err(UnsafeError::new(UnsafeErrorKind::SendPacket(ret)));        
    }

    let ret = avcodec_receive_frame(codec, frame);

    if ret != 0 {
        return Err(UnsafeError::new(UnsafeErrorKind::ReceiveFrame(ret)));
    }

    Ok(&mut *frame)
}

pub fn decode_raw_pkt_with_err_handling<'a>(context: &mut AVCodecContext, packet: &AVPacket, file: &mut File) -> Result<&'a mut AVFrame, UnsafeError> {
    unsafe {
        match decode_raw_packet(context, packet) {
            Ok(v) => Ok(v),
            Err(e) => {
                file.write(b"--- START ---");
                file.write(from_raw_parts(packet.data, packet.size as usize));
                file.write(b"--- END ---");
                Err(e)
            }
        }
    }
}

pub fn decode_packet<'a>(context: &mut AVCodecContext, packet: &AVPacket) -> Result<&'a mut AVFrame, UnsafeError> {
    unsafe {
        decode_raw_packet(context, packet)
    }
}