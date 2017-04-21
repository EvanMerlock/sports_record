use ffmpeg_sys::*;

pub fn make_av_rational(num: i32, den: i32) -> AVRational {
    unsafe {
        av_make_q(num, den)
    }
}