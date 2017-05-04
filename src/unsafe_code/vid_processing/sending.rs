use unsafe_code::UnsafeError;
use messenger_plus::stream::DualMessenger;
use uuid::Uuid;

use std::ffi::CString;
use std::net::TcpStream;
use std::fs::File;

use ffmpeg_sys::*;

use unsafe_code::{init_av, make_av_rational, CodecStorage};
use unsafe_code::vid_processing;
use unsafe_code::img_processing;
use unsafe_code::sws;
use unsafe_code::input;

pub fn send_video(stream: &mut DualMessenger<TcpStream>) -> Result<(), UnsafeError> {
    init_av();

    //CODEC ALLOCATION
    let encoding_context: &mut AVCodecContext = try!(vid_processing::create_encoding_context(AV_CODEC_ID_H264, 480, 640, make_av_rational(1, 25), make_av_rational(25, 1)));
    let decoding_context: &mut AVCodecContext = try!(vid_processing::create_decoding_context(AV_CODEC_ID_RAWVIDEO, 480, 640, make_av_rational(1, 25), make_av_rational(25, 1), AV_PIX_FMT_YUYV422));
    let jpeg_context: &mut AVCodecContext = try!(img_processing::create_jpeg_context(480, 640, make_av_rational(1, 25)));

    let mut context_storage: CodecStorage = CodecStorage::new(encoding_context, decoding_context, jpeg_context);

    //SWS ALLOCATION
    let sws_context = try!(sws::create_sws_context(480, 640, AV_PIX_FMT_YUYV422, AV_PIX_FMT_YUV420P));

    //INPUT ALLOCATION
    let input_format: &AVInputFormat = input::create_input_format(CString::new("v4l2").unwrap());
    let input_context: &mut AVFormatContext = try!(input::create_format_context(input_format, CString::new("/dev/video0").unwrap()));

    for pts in 0..100 {
        let ref mut packet = input::read_input(input_context);

        try!(transcode_packet(&mut context_storage, sws_context, &(*packet), pts, stream));

        input::unallocate_packet(packet);
    }

    try!(vid_processing::encode_null_frame(context_storage.encoding_context, stream));

    Ok(())

}

fn transcode_packet(contexts: &mut CodecStorage, sws_context: &mut SwsContext, packet: &AVPacket, pts: i64, stream: &mut DualMessenger<TcpStream>) -> Result<(), UnsafeError> {
    let raw_frame: &mut AVFrame = try!(vid_processing::decode_packet(contexts.decoding_context, packet));

    let scaled_frame: &mut AVFrame = try!(sws::change_pixel_format(raw_frame, sws_context, 32, pts));

    let file = try!(File::create(String::from("picture_") + Uuid::new_v4().to_string().as_ref() + String::from(".jpeg").as_ref()));
    try!(img_processing::write_frame_to_jpeg(contexts.jpeg_context, scaled_frame, file));

    try!(vid_processing::encode_frame(contexts.encoding_context, scaled_frame, stream));

    Ok(())
}