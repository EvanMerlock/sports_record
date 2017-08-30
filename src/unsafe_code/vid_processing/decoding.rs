
use std::ptr;
use std::ffi::CString;

use unsafe_code::{UnsafeError, UnsafeErrorKind, CodecContext, CodecId, AsRawPtr, Frame};
use config::stream_config::*;
use unsafe_code::codec::{DecodingCodecContext, DecodingCodec, Codec};

use ffmpeg_sys::*;
use libc;

unsafe fn allocate_decoding_codec_from_av_stream(stream_config: &mut AVStream) -> Result<DecodingCodecContext, UnsafeError> {
    let decoding_codec = Codec::new_decoder(CodecId::from((*stream_config.codecpar).codec_id));
    let mut decoding_context = CodecContext::new_codec_based_context(&decoding_codec);

    let ret = avcodec_parameters_to_context(&mut *decoding_context, stream_config.codecpar);
    if ret < 0 {
        return Err(UnsafeError::new(UnsafeErrorKind::OpenDecoder(ret)));
    }


    if decoding_context.codec_id == AV_CODEC_ID_H264 {
        av_opt_set((&mut *decoding_context as *mut AVCodecContext) as *mut libc::c_void, CString::new("preset").unwrap().as_ptr(), CString::new("ultrafast").unwrap().as_ptr(), 0);
        av_opt_set((&mut *decoding_context as *mut AVCodecContext) as *mut libc::c_void, CString::new("crf").unwrap().as_ptr(), CString::new("28").unwrap().as_ptr(), 0);
    }

    let dec_final = try!(decoding_context.open_decoding(decoding_codec));

    Ok(dec_final)
}

pub fn create_decoding_context_from_av_stream(stream: &mut AVStream) -> Result<DecodingCodecContext, UnsafeError> {
    unsafe {
        allocate_decoding_codec_from_av_stream(stream)
    }
}

unsafe fn decode_raw_packet<'a>(codec: &mut DecodingCodecContext, packet: &AVPacket) -> Result<Frame, UnsafeError> {
    let frame = av_frame_alloc();

    let ret = avcodec_send_packet(codec.as_mut_ptr(), packet);

    if ret != 0 {
        return Err(UnsafeError::new(UnsafeErrorKind::SendPacket(ret)));        
    }

    let ret = avcodec_receive_frame(codec.as_mut_ptr(), frame);

    if ret != 0 {
        return Err(UnsafeError::new(UnsafeErrorKind::ReceiveFrame(ret)));
    }

    Ok(Frame::from(frame))
}

pub fn decode_packet<'a>(context: &mut DecodingCodecContext, packet: &AVPacket) -> Result<Frame, UnsafeError> {
    unsafe {
        decode_raw_packet(context, packet)
    }
}