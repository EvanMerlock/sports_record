use std::marker::{Send, Sync};
use std::convert::{From, Into};
use std::default::Default;

use ffmpeg_sys::*;

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub struct Rational(i32, i32);

unsafe impl Send for Rational {}
unsafe impl Sync for Rational {}

impl Rational {
    pub fn new(num: i32, den: i32) -> Rational {
        Rational(num, den)
    }
}

impl From<AVRational> for Rational {
    fn from(rat: AVRational) -> Rational {
        Rational(rat.num, rat.den)
    }
}

impl Into<AVRational> for Rational {
    fn into(self) -> AVRational {
        make_av_rational(self.0, self.1)
    }
}

impl Default for Rational {
    fn default() -> Self {
        Rational::new(0, 0)
    }
}

pub fn make_av_rational(num: i32, den: i32) -> AVRational {
    unsafe {
        av_make_q(num, den)
    }
}