use ffmpeg_sys::*;

pub fn make_av_rational(num: i32, den: i32) -> AVRational {
    unsafe {
        av_make_q(num, den)
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