use std::ptr;
use std::ffi::CString;

use unsafe_code::{UnsafeError, UnsafeErrorKind, CodecContext, Rational};
use unsafe_code::packet::Packet;

use ffmpeg_sys::*;
use libc;

unsafe fn allocate_encoding_codec(codec_type: AVCodecID, height: i32, width: i32, time_base: Rational, gop_size: i32, max_b_frames: i32) -> Result<(*mut AVCodec, *mut AVCodecContext), UnsafeError> {

    let codec_ptr: *mut AVCodec = avcodec_find_encoder(codec_type);
    let encoding_context_ptr: *mut AVCodecContext = avcodec_alloc_context3(codec_ptr);

    let ref mut encoding_context: AVCodecContext = *encoding_context_ptr;

    encoding_context.height = height;
    encoding_context.width = width;

    encoding_context.time_base = time_base.into();

    encoding_context.gop_size = gop_size;
    encoding_context.max_b_frames = max_b_frames;
    encoding_context.pix_fmt = AV_PIX_FMT_YUV420P;

    if codec_type == AV_CODEC_ID_H264 {
        av_opt_set((encoding_context as *mut AVCodecContext) as *mut libc::c_void, CString::new("preset").unwrap().as_ptr(), CString::new("ultrafast").unwrap().as_ptr(), 0);
        av_opt_set((encoding_context as *mut AVCodecContext) as *mut libc::c_void, CString::new("crf").unwrap().as_ptr(), CString::new("28").unwrap().as_ptr(), 0);
    }

    let ret = avcodec_open2(encoding_context_ptr, codec_ptr, ptr::null_mut());
    if ret < 0 {
        return Err(UnsafeError::new(UnsafeErrorKind::OpenEncoder(ret)));
    }

    Ok((codec_ptr, encoding_context_ptr))

}

pub fn create_encoding_context(codec_type: AVCodecID, height: i32, width: i32, time_base: Rational, gop_size: i32, max_b_frames: i32) -> Result<CodecContext, UnsafeError> {
    unsafe {
        match allocate_encoding_codec(codec_type, height, width, time_base, gop_size, max_b_frames) {
            Ok((_, context)) => Ok(CodecContext::from(context)),
            Err(e) => Err(e),
        }
    }
}

unsafe fn encode_raw_frame(codec: &mut AVCodecContext, frame: *mut AVFrame) -> Result<Vec<Packet>, UnsafeError> {    
    let ret = avcodec_send_frame(codec, frame);
    let mut vec = Vec::new();

    if ret < 0 {
        return Err(UnsafeError::new(UnsafeErrorKind::SendFrame(ret)));
    }

    while ret >= 0 {
        let packet = av_packet_alloc();
        let ret = avcodec_receive_packet(codec, packet);

        if ret == -11 || ret == AVERROR_EOF {
            return Ok(vec);
        } else if ret < 0 {
            return Err(UnsafeError::new(UnsafeErrorKind::ReceivePacket(ret)));
        }

        vec.push(Packet::from(Box::from_raw(packet)));
        // let res = stream.write(from_raw_parts((*packet).data, (*packet).size as usize));

    }

    Ok(vec)

}

pub fn encode_frame<'a>(context: &mut AVCodecContext, frame: &mut AVFrame) -> Result<Vec<Packet>, UnsafeError> {
    unsafe {
        encode_raw_frame(context, frame)
    }
}

pub fn encode_null_frame<'a>(context: &mut AVCodecContext) -> Result<Vec<Packet>, UnsafeError> {
    unsafe {
        encode_raw_frame(context, ptr::null_mut())
    }
}