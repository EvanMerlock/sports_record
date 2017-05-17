use ffmpeg_sys::*;

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