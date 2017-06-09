use time::{Duration, PreciseTime};

use std::borrow::BorrowMut;
use std::thread;
use std::thread::JoinHandle;
use std::sync::mpsc::{channel, Sender, Receiver};

use std::ffi::CString;

use ffmpeg_sys::*;

use unsafe_code::{init_av, CodecStorage, UnsafeError, UnsafeErrorKind, Rational};
use unsafe_code::vid_processing;
use unsafe_code::sws;
use unsafe_code::sws::SWSContext;
use unsafe_code::input;
use unsafe_code::packet::{Packet, DataPacket};
use config::stream_config::StreamConfiguration;
use networking::NetworkPacket;

enum PacketMessage {
    Packet(Packet),
    Flush,
}

pub fn send_video<'a>(stream: Sender<NetworkPacket>) -> Result<(), UnsafeError> {
    init_av();

    //INPUT ALLOCATION
    let input_format: &AVInputFormat = input::create_input_format(CString::new("v4l2").unwrap());
    let input_context: &mut AVFormatContext = try!(input::create_format_context(input_format, CString::new("/dev/video0").unwrap()));

    //Grab the stream from the input context
    let opt = input::find_input_stream(input_context, 0);

    if let Some(in_str) = opt {
        let (packet_tx, packet_rx) = channel();
        let (context_storage, sws_context) = try!(generate_contexts(in_str));

        let output_stream_configuration = StreamConfiguration::from(&context_storage.encoding_context);

        let _ = stream.send(NetworkPacket::JSONPayload(output_stream_configuration));
        let render_thread_handle = spawn_thread(context_storage, sws_context, stream, packet_rx);

        let start_time = PreciseTime::now();
        let mut packets_read = 0;
        while start_time.to(PreciseTime::now()) <= Duration::seconds(5) {
            let packet = input::read_input(input_context);
            let _ = packet_tx.send(PacketMessage::Packet(packet));
            packets_read = packets_read + 1;
        }
        
        println!("Read {} packets, now sending flush signal", packets_read);
        let _ = packet_tx.send(PacketMessage::Flush);

        let _ = render_thread_handle.join();

        Ok(())
    } else {
        Err(UnsafeError::new(UnsafeErrorKind::OpenInput(1000)))
    }

}

fn generate_contexts<'a>(stream: &mut AVStream) -> Result<(CodecStorage, SWSContext), UnsafeError> {
    //CODEC ALLOCATION
    let decoding_context = try!(vid_processing::create_decoding_context_from_av_stream(stream));
    let encoding_context = try!(vid_processing::create_encoding_context(AV_CODEC_ID_H264, 480, 640, Rational::new(1, 30), decoding_context.gop_size, decoding_context.max_b_frames));
    let context_storage: CodecStorage = CodecStorage::new(encoding_context, decoding_context);

    // SWS ALLOCATION
    let sws_context = try!(sws::create_sws_context(480, 640, AV_PIX_FMT_YUYV422, AV_PIX_FMT_YUV420P));

    Ok((context_storage, sws_context))
}

fn spawn_thread(mut context_storage: CodecStorage, mut sws_context: SWSContext, stream: Sender<NetworkPacket>, packet_rx: Receiver<PacketMessage>) -> JoinHandle<i64> {
    thread::spawn(move || {
        let mut time = 0;
        for item in packet_rx.iter() {
            match item {
                PacketMessage::Packet(p) => {
                    let conv_pkt_attempt = transcode_packet(&mut context_storage, &mut sws_context, p, time);
                    if let Ok(conv_pkt) = conv_pkt_attempt {
                        let _ = stream.send(conv_pkt);
                        time = time + 1;
                    } else {
                        break;
                    }
                },
                PacketMessage::Flush => { break; }
            }
        }
        println!("flushing packets");
        let null_pkt_attempt = vid_processing::encode_null_frame(context_storage.encoding_context.borrow_mut());
        if let Ok(null_pkt) = null_pkt_attempt {
            println!("sending null pkt of len {}", null_pkt.len());
            let _ = stream.send(NetworkPacket::PacketStream(null_pkt.into_iter().map(|x: Packet| DataPacket::from(x)).collect()));
        } else {
            println!("error sending null pkt");
        }
        println!("finished sending");
        time
    })
}

fn transcode_packet<'a>(contexts: &mut CodecStorage, sws_context: &mut SWSContext, packet: Packet, frame_loc: i64) -> Result<NetworkPacket, UnsafeError> {
    let raw_frame: &mut AVFrame = try!(vid_processing::decode_packet(contexts.decoding_context.borrow_mut(), &packet));

    let scaled_frame: &mut AVFrame = try!(sws::change_pixel_format(raw_frame, sws_context.borrow_mut(), 32, frame_loc));

    let pkts = try!(vid_processing::encode_frame(contexts.encoding_context.borrow_mut(), scaled_frame));
    Ok(NetworkPacket::PacketStream(pkts.into_iter().map(|x: Packet| DataPacket::from(x)).collect()))
}