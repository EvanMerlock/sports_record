use unsafe_code::packet::Packet;

use ffmpeg_sys::*;

unsafe fn grab_from_input<'a>(input: &mut AVFormatContext) -> *mut AVPacket {
    let pkt = av_packet_alloc();

    av_read_frame(input, pkt);

    pkt
}

pub fn read_input<'a>(input: &mut AVFormatContext) -> Packet {
    unsafe {
        Packet::from(grab_from_input(input))
    }
}

pub fn unallocate_packet(pkt: &mut AVPacket) {
    unsafe {
        av_packet_unref(pkt)
    }
}