use unsafe_code::{UnsafeError, UnsafeErrorKind};

use std::ptr;

use ffmpeg_sys::*;

unsafe fn allocate_decoding_codec(decoder_id: AVCodecID, height: i32, width: i32, time_base: AVRational, framerate: AVRational, pixel_fmt: AVPixelFormat) -> Result<(*mut AVCodec, *mut AVCodecContext), UnsafeError> {

    let codec_ptr: *mut AVCodec = avcodec_find_decoder(decoder_id);
    let decoding_context_ptr: *mut AVCodecContext = avcodec_alloc_context3(codec_ptr);

    let ref mut decoding_context: AVCodecContext = *decoding_context_ptr;

    decoding_context.height = height;
    decoding_context.width = width;

    decoding_context.time_base = time_base;
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

pub fn create_decoding_context<'a>(decoder_id: AVCodecID, height: i32, width: i32, time_base: AVRational, framerate: AVRational, pixel_fmt: AVPixelFormat) -> Result<&'a mut AVCodecContext, UnsafeError> {
    unsafe {
        match allocate_decoding_codec(decoder_id, height, width, time_base, framerate, pixel_fmt) {
            Ok((_, context)) => Ok(&mut *context),
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