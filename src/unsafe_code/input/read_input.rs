use ffmpeg_sys::*;

pub fn unallocate_packet(pkt: &mut AVPacket) {
    unsafe {
        av_packet_unref(pkt)
    }
}