use std::borrow::BorrowMut;
use std::thread;
use std::thread::JoinHandle;
use std::sync::mpsc::{channel, Sender, Receiver};

use std::ffi::CString;

use client::ClientStatusFlag;

use unsafe_code::{init_av, CodecStorage, UnsafeError, UnsafeErrorKind, Rational, CodecId, Frame};
use unsafe_code::vid_processing;
use unsafe_code::format::{FormatContext, InputContext, Stream};
use unsafe_code::sws;
use unsafe_code::sws::SWSContext;
use unsafe_code::{Packet, DataPacket};
use config::stream_config::StreamConfiguration;
use networking::NetworkPacket;

use ffmpeg_sys::*;

enum PacketMessage {
    Packet(Packet),
    Flush,
}

pub fn send_video(message_transfer: Receiver<ClientStatusFlag>, stream: Sender<NetworkPacket>) -> Result<i64, UnsafeError> {
    let client_flag = try!(message_transfer.recv());

    if client_flag != ClientStatusFlag::StartRecording {
        return Ok(0);
    }
    
    init_av();

    //INPUT ALLOCATION
    let input_format: &AVInputFormat = InputContext::create_input_format(CString::new("v4l2").unwrap());
    let mut input_context: InputContext = FormatContext::new_input(input_format, CString::new("/dev/video0").unwrap())?;

    //Grab the stream from the input context
    let opt = input_context.find_input_stream(0);

    if let Some(mut in_str) = opt {
        let (packet_tx, packet_rx) = channel();
        let context_storage = try!(generate_contexts(&mut in_str));

        let output_stream_configuration = StreamConfiguration::from(context_storage.encoding_context.as_ref());

        let _ = stream.send(NetworkPacket::JSONPayload(output_stream_configuration));
        let render_thread_handle = spawn_thread(context_storage, stream, packet_rx);

        let mut packets_read = 0;
        loop {
            if let Ok(msg) = message_transfer.try_recv() {
                break;
            }
            let mut packet = input_context.read_input();
            packet.pts = packets_read;
            let _ = packet_tx.send(PacketMessage::Packet(packet));
            packets_read = packets_read + 1;
        }
        
        println!("Read {} packets, now sending flush signal", packets_read);
        let _ = packet_tx.send(PacketMessage::Flush);

        let _ = render_thread_handle.join();

        Ok(packets_read)
    } else {
        Err(UnsafeError::new(UnsafeErrorKind::OpenInput(1000)))
    }

}

fn generate_contexts(stream: &mut Stream) -> Result<CodecStorage, UnsafeError> {
    //CODEC ALLOCATION
    let decoding_context = try!(vid_processing::create_decoding_context_from_av_stream(stream));
    let encoding_context = vid_processing::create_encoding_context(
        CodecId::from(AV_CODEC_ID_H264), 
        480, 640, 
        Rational::new(1, 30), 
        decoding_context.as_ref().gop_size, 
        decoding_context.as_ref().max_b_frames
    )?;

    // SWS ALLOCATION
    let sws_context = try!(sws::create_sws_context(480, 640, AV_PIX_FMT_YUYV422, AV_PIX_FMT_YUV420P));
    let context_storage: CodecStorage = CodecStorage::new(encoding_context, decoding_context, sws_context);


    Ok(context_storage)
}

fn spawn_thread(mut context_storage: CodecStorage, stream: Sender<NetworkPacket>, packet_rx: Receiver<PacketMessage>) -> JoinHandle<i64> {
    thread::spawn(move || {
        let mut time = 0;
        for item in packet_rx.iter() {
            match item {
                PacketMessage::Packet(p) => {
                    let conv_pkt_attempt = transcode_packet(&mut context_storage, p, time);
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
        let _ = stream.send(NetworkPacket::PayloadEnd);
        time
    })
}

fn transcode_packet(contexts: &mut CodecStorage, packet: Packet, frame_loc: i64) -> Result<NetworkPacket, UnsafeError> {
    let raw_frame: Frame = try!(vid_processing::decode_packet(contexts.decoding_context.borrow_mut(), &packet));

    let scaled_frame: Frame = try!(sws::change_pixel_format(raw_frame, contexts.sws_context.borrow_mut(), 32, frame_loc));
    println!("current frame pts: {}", scaled_frame.pts);

    let pkts = try!(vid_processing::encode_frame(contexts.encoding_context.borrow_mut(), scaled_frame));
    Ok(NetworkPacket::PacketStream(pkts.into_iter().map(|x: Packet| DataPacket::from(x)).collect()))
}