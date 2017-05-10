use uuid::Uuid;
use time::{Duration, PreciseTime};

use std::io::Write;
use std::borrow::{Borrow, BorrowMut};
use std::thread;
use std::sync::mpsc::{channel, Sender, Receiver};

use std::ffi::CString;
use std::fs::File;

use ffmpeg_sys::*;

use unsafe_code::{init_av, make_av_rational, CodecStorage, UnsafeError, UnsafeErrorKind};
use unsafe_code::vid_processing;
use unsafe_code::img_processing;
use unsafe_code::sws;
use unsafe_code::input;
use unsafe_code::packet::Packet;
use config::stream_config::StreamConfiguration;

use std::slice::from_raw_parts;

enum PacketMessage {
    Packet(Packet),
    Flush,
}

pub fn send_video<'a>(stream: Sender<Vec<Packet>>) -> Result<(), UnsafeError> {
    init_av();

    //INPUT ALLOCATION
    let input_format: &AVInputFormat = input::create_input_format(CString::new("v4l2").unwrap());
    let input_context: &mut AVFormatContext = try!(input::create_format_context(input_format, CString::new("/dev/video0").unwrap()));

    //Grab the stream from the input context
    let opt = input::find_input_stream(input_context, 0);

    match opt {
        Some(in_str) => {
            //Thread allocation
            let in_str_config = StreamConfiguration::from(in_str);
            in_str.time_base = make_av_rational(1, 30);

            let (packet_tx, packet_rx) = channel();

            //CODEC ALLOCATION
            let decoding_context = vid_processing::create_decoding_context_from_stream_configuration(in_str_config).expect("failure to open decoding context");
            let encoding_context = vid_processing::create_encoding_context(AV_CODEC_ID_H264, 480, 640, make_av_rational(1, 30), decoding_context.gop_size, decoding_context.max_b_frames).expect("failure to open encoding context");
            let jpeg_context = img_processing::create_jpeg_context(480, 640, make_av_rational(1, 30)).expect("failure to open JPEG context");
            let mut context_storage: CodecStorage = CodecStorage::new(encoding_context, decoding_context, jpeg_context);

            // SWS ALLOCATION
            let sws_context = sws::create_sws_context(480, 640, AV_PIX_FMT_YUYV422, AV_PIX_FMT_YUV420P).expect("failure to open SWS context");

            thread::spawn(move || {
                let mut time = 0;
                for item in packet_rx.iter() {
                    match item {
                        PacketMessage::Packet(p) => {
                            let conv_pkt_attempt = transcode_packet(&mut context_storage, sws_context, &p, time);
                            match conv_pkt_attempt {
                                Ok(conv_pkt) => {
                                    stream.send(conv_pkt);
                                    time = time + 1;
                                },
                                Err(e) => { break; }
                            }
                        },
                        PacketMessage::Flush => { break; }
                    }
                }
                println!("flushing packets");
                let null_pkt_attempt = vid_processing::encode_null_frame(context_storage.encoding_context.borrow_mut());
                match null_pkt_attempt {
                    Ok(null_pkt) => {
                        println!("sending null pkt of len {}", null_pkt.len());
                        stream.send(null_pkt);
                    },
                    Err(e) => { println!("err {}", e); }
                }
                println!("finished sending");
            });

            let start_time = PreciseTime::now();
            while start_time.to(PreciseTime::now()) < Duration::seconds(5) {
                let packet = input::read_input(input_context);
                packet_tx.send(PacketMessage::Packet(packet));
            }

            println!("sending flush signal");
            packet_tx.send(PacketMessage::Flush);

            Ok(())
        },
        None => {
            Err(UnsafeError::new(UnsafeErrorKind::OpenInput(1000)))
        }
    }

}

fn transcode_packet<'a>(contexts: &mut CodecStorage, sws_context: &mut SwsContext, packet: &Packet, frame_loc: i64) -> Result<Vec<Packet>, UnsafeError> {
    let raw_frame: &mut AVFrame = try!(vid_processing::decode_packet(contexts.decoding_context.borrow_mut(), &packet));

    let scaled_frame: &mut AVFrame = try!(sws::change_pixel_format(raw_frame, sws_context.borrow_mut(), 32, frame_loc));

    let file = try!(File::create(String::from("picture_") + Uuid::new_v4().to_string().as_ref() + String::from(".jpeg").as_ref()));
    try!(img_processing::write_frame_to_jpeg(contexts.jpeg_context.borrow_mut(), scaled_frame, file));

    let pkts = try!(vid_processing::encode_frame(contexts.encoding_context.borrow_mut(), scaled_frame));
    Ok(pkts)
}

pub fn write_to_stream(mut frames: Vec<Packet>, writer: &mut Write) -> Result<(), UnsafeError> {
    for mut frame in frames.drain(0..) {
        unsafe {
            try!(writer.write(from_raw_parts(frame.data, frame.size as usize)));
            av_packet_unref(&mut *frame as *mut AVPacket);
        }
    }
    Ok(())
}