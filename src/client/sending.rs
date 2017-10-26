use std::thread;
use std::thread::JoinHandle;
use std::sync::mpsc::{channel, Sender, Receiver, TryRecvError};
use std::sync::Arc;

use std::ffi::CString;

use client::ClientStatusFlag;

use config::client_configuration::CameraConfiguration;
use unsafe_code::{init_av, CodecStorage, UnsafeError, UnsafeErrorKind, Rational, CodecId, Frame};
use unsafe_code::format::{FormatContext, InputContext, Stream};
use unsafe_code::sws::SWSContext;
use unsafe_code::img_processing;
use unsafe_code::{Packet, DataPacket, EncodingCodecContext, DecodingCodecContext};
use config::stream_config::StreamConfiguration;
use networking::NetworkPacket;

use ffmpeg_sys::*;

enum PacketMessage {
    Packet(Packet),
    Flush,
}

pub fn send_video(camera_config: CameraConfiguration, message_transfer: Receiver<ClientStatusFlag>, stream: Sender<NetworkPacket>, jpeg_sender: Sender<Arc<Vec<u8>>>) -> Result<(), UnsafeError> {  
    init_av();

    //INPUT ALLOCATION
    let input_format: &mut AVInputFormat = InputContext::create_input_format(camera_config.get_input_type());
    let mut input_context: InputContext = FormatContext::new_input(input_format, camera_config.get_camera_location())?;

    //Grab the stream from the input context
    let opt = input_context.find_input_stream(0);

    if let Some(mut in_str) = opt {
        let context_storage = try!(generate_contexts(&mut in_str));
        let output_stream_configuration = StreamConfiguration::from(<EncodingCodecContext as AsRef<AVCodecContext>>::as_ref(&context_storage.encoding_context));
        let _ = stream.send(NetworkPacket::JSONPayload(output_stream_configuration));

        let mut sender = jpeg_sender;

        let mut currently_recording = false;
        loop {
            let (packet_tx, packet_rx) = channel();
            let render_thread_handle = spawn_thread(context_storage.clone(), stream.clone(), packet_rx, sender);
            let mut packets_read = 0;
            loop {
                match message_transfer.try_recv() {
                    Ok(ref m) if m == &ClientStatusFlag::StopRecording => {
                        break;
                    },
                    Ok(ref m) if m == &ClientStatusFlag::StartRecording => {
                        currently_recording = true;
                    },
                    Err(ref e) if !(e == &TryRecvError::Empty) => {
                        break;
                    }
                    _ => {},
                }
                if currently_recording {
                    let mut packet = input_context.read_input();
                    packet.pts = packets_read;
                    let _ = packet_tx.send(PacketMessage::Packet(packet));
                    packets_read = packets_read + 1;                    
                }
            }
            currently_recording = false;
            println!("Read {} packets, now sending flush signal", packets_read);
            let _ = packet_tx.send(PacketMessage::Flush);

            sender = render_thread_handle.join().expect("couldn't join to thread");
        }
        Ok(())
    } else {
        Err(UnsafeError::new(UnsafeErrorKind::OpenInput(1000)))
    }

}

fn generate_contexts(stream: &mut Stream) -> Result<CodecStorage, UnsafeError> {
    //CODEC ALLOCATION
    let decoding_context = try!(DecodingCodecContext::create_decoding_context_from_av_stream(stream));

    let stream_configuration = StreamConfiguration::from(stream as &_);

    let encoding_context = EncodingCodecContext::create_encoding_context(
        CodecId::from(AVCodecID::AV_CODEC_ID_H264), 
        stream_configuration.height, stream_configuration.width, 
        Rational::new(1, 30),
        0, 0
    )?;

    let png_context = EncodingCodecContext::create_png_context(
        stream_configuration.height, stream_configuration.width,
        Rational::new(1, 30)
    )?;

    let png_sws_context = SWSContext::new(stream_configuration.height, stream_configuration.width, AVPixelFormat::AV_PIX_FMT_YUV420P, AVPixelFormat::AV_PIX_FMT_RGB24)?;

    // SWS ALLOCATION
    let sws_context = try!(SWSContext::new(stream_configuration.height, stream_configuration.width, *stream_configuration.pix_fmt, AVPixelFormat::AV_PIX_FMT_YUV420P));
    let context_storage: CodecStorage = CodecStorage::new(encoding_context, decoding_context, png_context, sws_context, png_sws_context);


    Ok(context_storage)
}

fn spawn_thread(mut context_storage: CodecStorage, stream: Sender<NetworkPacket>, packet_rx: Receiver<PacketMessage>, png_sender: Sender<Arc<Vec<u8>>>) -> JoinHandle<Sender<Arc<Vec<u8>>>> {
    thread::spawn(move || {
        let mut time = 0;
        for item in packet_rx.iter() {
            match item {
                PacketMessage::Packet(packet) => {
                    let conv_pkt_attempt = transcode_packet(&mut context_storage, &png_sender, packet, time);
                    if let Ok(conv_pkt) = conv_pkt_attempt {
                        let _ = stream.send(conv_pkt);
                        time = time + 1;
                    } else {
                        println!("failed to conv pkt: {:?}", conv_pkt_attempt);
                        break;
                    }
                },
                PacketMessage::Flush => { break; }
            }
        }
        println!("flushing packets");
        let null_pkt_attempt = context_storage.encoding_context.encode_null_frame();
        if let Ok(null_pkt) = null_pkt_attempt {
            println!("sending null pkt of len {}", null_pkt.len());
            let _ = stream.send(NetworkPacket::PacketStream(null_pkt.into_iter().map(|x: Packet| DataPacket::from(x)).collect()));
        } else {
            println!("error sending null pkt");
        }
        println!("finished sending");
        let _ = stream.send(NetworkPacket::PayloadEnd);
        png_sender
    })
}

fn transcode_packet(contexts: &mut CodecStorage, png_sender: &Sender<Arc<Vec<u8>>>, packet: Packet, frame_loc: i64) -> Result<NetworkPacket, UnsafeError> {
    let mut raw_frame: Frame = try!(contexts.decoding_context.decode_packet(&packet));

    let mut scaled_frame: Frame = contexts.sws_context.change_pixel_format(&mut raw_frame, 32, frame_loc)?;
    let png_frame: Frame = contexts.png_sws_context.change_pixel_format(&mut scaled_frame, 32, frame_loc)?;
    println!("current frame pts: {}", scaled_frame.pts);

    match contexts.png_context.encode_png_frame(&png_frame) {
        Ok(e) => { png_sender.send(Arc::new(e)); },
        Err(e) => println!("{:?}", e),
    }


    let pkts = try!(contexts.encoding_context.encode_frame(scaled_frame));
    Ok(NetworkPacket::PacketStream(pkts.into_iter().map(|x: Packet| DataPacket::from(x)).collect()))
}