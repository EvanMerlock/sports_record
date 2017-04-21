use ffmpeg_sys::*;

unsafe fn grab_from_input<'a>(input: &mut AVFormatContext) -> &'a mut AVPacket {
    let pkt = av_packet_alloc();

    av_read_frame(input, pkt);

    &mut *pkt
}

pub fn read_input<'a>(input: &mut AVFormatContext) -> &'a mut AVPacket {
    unsafe {
        grab_from_input(input)
    }
}

pub fn unallocate_packet(pkt: &mut AVPacket) {
    unsafe {
        av_packet_unref(pkt)
    }
}