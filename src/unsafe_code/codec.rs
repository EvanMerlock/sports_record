use ffmpeg_sys::*;

pub struct CodecStorage<'a> {
    pub encoding_context: &'a mut AVCodecContext,
    pub decoding_context: &'a mut AVCodecContext,
    pub jpeg_context: &'a mut AVCodecContext,
}

impl<'a> CodecStorage<'a> {

    pub fn new<'b>(enc: &'b mut AVCodecContext, dec: &'b mut AVCodecContext, jpeg: &'b mut AVCodecContext) -> CodecStorage<'b> {
        CodecStorage {
            encoding_context: enc,
            decoding_context: dec,
            jpeg_context: jpeg,
        }
    }

}

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