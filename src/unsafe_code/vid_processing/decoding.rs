use unsafe_code::{UnsafeError, UnsafeErrorKind, CodecContext, Rational};
use config::stream_config::*;

use std::ptr;

use ffmpeg_sys::*;

unsafe fn allocate_decoding_codec(decoder_id: AVCodecID, height: i32, width: i32, framerate: AVRational, pixel_fmt: AVPixelFormat) -> Result<(*mut AVCodec, *mut AVCodecContext), UnsafeError> {

    let codec_ptr: *mut AVCodec = avcodec_find_decoder(decoder_id);
    let decoding_context_ptr: *mut AVCodecContext = avcodec_alloc_context3(codec_ptr);

    let ref mut decoding_context: AVCodecContext = *decoding_context_ptr;

    decoding_context.height = height;
    decoding_context.width = width;

    decoding_context.framerate = framerate;

    decoding_context.gop_size = 10;
    decoding_context.max_b_frames = 1;
    decoding_context.pix_fmt = pixel_fmt;

    let ret = avcodec_open2(decoding_context_ptr, codec_ptr, ptr::null_mut());
    if ret < 0 {
        return Err(UnsafeError::new(UnsafeErrorKind::OpenDecoder(ret)));
    }

    Ok((codec_ptr, decoding_context_ptr))

}

unsafe fn allocate_decoding_codec_from_stream_config(stream_config: StreamConfiguration) -> Result<(*mut AVCodec, *mut AVCodecContext), UnsafeError> {

    let codec_ptr: *mut AVCodec = avcodec_find_decoder(stream_config.codec_id);
    let decoding_context_ptr: *mut AVCodecContext = avcodec_alloc_context3(codec_ptr);

    let ref mut decoding_context: AVCodecContext = *decoding_context_ptr;

    decoding_context.bit_rate = stream_config.bit_rate;
    decoding_context.height = stream_config.height;
    decoding_context.width = stream_config.width;

    decoding_context.framerate = stream_config.frame_rate.into();

    // decoding_context.gop_size = stream_config.gop_size;
    // decoding_context.max_b_frames = stream_config.max_b_frames;

    decoding_context.gop_size = 0;
    decoding_context.max_b_frames = 0;

    decoding_context.pix_fmt = stream_config.pix_fmt;

    let ret = avcodec_open2(decoding_context_ptr, codec_ptr, ptr::null_mut());
    if ret < 0 {
        return Err(UnsafeError::new(UnsafeErrorKind::OpenDecoder(ret)));
    }

    Ok((codec_ptr, decoding_context_ptr))
}

pub fn create_decoding_context(decoder_id: AVCodecID, height: i32, width: i32, framerate: AVRational, pixel_fmt: AVPixelFormat) -> Result<CodecContext, UnsafeError> {
    unsafe {
        match allocate_decoding_codec(decoder_id, height, width, framerate, pixel_fmt) {
            Ok((_, context)) => Ok(CodecContext::from(context)),
            Err(e) => Err(e),
        }
    }
}

pub fn create_decoding_context_from_stream_configuration(stream: StreamConfiguration) -> Result<CodecContext, UnsafeError> {
    unsafe {
        match allocate_decoding_codec_from_stream_config(stream) {
            Ok((_, context)) => Ok(CodecContext::from(context)),
            Err(e) => Err(e),
        }
    }
}

unsafe fn decode_raw_packet<'a>(codec: &mut AVCodecContext, packet: &AVPacket) -> Result<&'a mut AVFrame, UnsafeError> {
    let frame = av_frame_alloc();

    let ret = avcodec_send_packet(codec, packet);

    if ret < 0 {
        return Err(UnsafeError::new(UnsafeErrorKind::SendPacket(ret)));
    }

    let ret = avcodec_receive_frame(codec, frame);

    if ret < 0 {
        return Err(UnsafeError::new(UnsafeErrorKind::ReceivePacket(ret)));
    }

    Ok(&mut *frame)
}

pub fn decode_packet<'a>(context: &mut AVCodecContext, packet: &AVPacket) -> Result<&'a mut AVFrame, UnsafeError> {
    unsafe {
        decode_raw_packet(context, packet)
    }
}