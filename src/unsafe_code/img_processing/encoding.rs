use unsafe_code::{UnsafeError, UnsafeErrorKind, CodecContext};
use unsafe_code::img_processing::magick;

use std::io::Write;
use std::fs::File;

use std::slice::from_raw_parts;
use std::ptr;

use ffmpeg_sys::*;

unsafe fn allocate_jpeg_codec(height: i32, width: i32, time_base: AVRational) -> Result<(*mut AVCodec, *mut AVCodecContext), UnsafeError> {

    let codec_ptr: *mut AVCodec = avcodec_find_encoder(AV_CODEC_ID_JPEG2000);
    let jpeg_context_ptr: *mut AVCodecContext = avcodec_alloc_context3(codec_ptr);

    let ref mut jpeg_context: AVCodecContext = *jpeg_context_ptr;

    jpeg_context.height = height;
    jpeg_context.width = width;

    jpeg_context.time_base = time_base;

    jpeg_context.pix_fmt = AV_PIX_FMT_YUV420P;

    let ret = avcodec_open2(jpeg_context_ptr, codec_ptr, ptr::null_mut());
    if ret < 0 {
        return Err(UnsafeError::new(UnsafeErrorKind::OpenEncoder(ret)));
    }

    Ok((codec_ptr, jpeg_context_ptr))


}

pub fn create_jpeg_context(height: i32, width: i32, time_base: AVRational) -> Result<CodecContext, UnsafeError> {
    unsafe {
        match allocate_jpeg_codec(height, width, time_base) {
            Ok((_, context)) => Ok(CodecContext::from(context)),
            Err(e) => Err(e),
        }
    }
}

unsafe fn encode_jpeg_frame(codec: &mut AVCodecContext, frame: &AVFrame, mut file: File) -> Result<(), UnsafeError> {
    let packet = av_packet_alloc();
    let ret = avcodec_send_frame(codec, frame);

    if ret < 0 {
        return Err(UnsafeError::new(UnsafeErrorKind::SendFrame(ret)));
    }

    let ret = avcodec_receive_packet(codec, packet);

    if ret == -11 || ret == AVERROR_EOF {
        return Ok(());
    } else if ret < 0 {
        return Err(UnsafeError::new(UnsafeErrorKind::ReceivePacket(ret)));
    }

    let img_vec = from_raw_parts((*packet).data, (*packet).size as usize).to_vec();
    let img_vec = try!(magick::convert_colorspace(img_vec));


    let _ = file.write(img_vec.as_slice());
    av_packet_unref(packet);

    Ok(())
}

pub fn write_frame_to_jpeg(codec: &mut AVCodecContext, frame: &AVFrame, mut file: File) -> Result<(), UnsafeError> {
    unsafe {
        encode_jpeg_frame(codec, frame, file)
    }
}