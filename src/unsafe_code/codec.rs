use ffmpeg_sys::*;

pub struct CodecStorage {
    pub encoding_context: Box<AVCodecContext>,
    pub decoding_context: Box<AVCodecContext>,
    pub jpeg_context: Box<AVCodecContext>,
}

impl CodecStorage {

    pub fn new(enc: Box<AVCodecContext>, dec: Box<AVCodecContext>, jpeg: Box<AVCodecContext>) -> CodecStorage {
        CodecStorage {
            encoding_context: enc,
            decoding_context: dec,
            jpeg_context: jpeg,
        }
    }

}

unsafe impl Send for CodecStorage {}

unsafe fn register_av() {
    av_register_all();
    avdevice_register_all();
    avcodec_register_all();
}

pub fn init_av() {
    unsafe {
        register_av();
    }
}