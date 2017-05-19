use std::marker::{Send, Sync};
use std::fmt;
use std::convert::{From};
use std::ops::{Deref};
use serde::{Serialize, Deserialize, Serializer, Deserializer};
use serde::de::{Visitor, EnumAccess};

use ffmpeg_sys::*;

#[derive(Debug)]
pub struct PixelFormat(AVPixelFormat);

unsafe impl Send for PixelFormat {}
unsafe impl Sync for PixelFormat {}

impl PixelFormat {
    fn to_static_str(&self) -> &'static str {
        match self.0 {
            AV_PIX_FMT_NONE    => "AV_PIX_FMT_NONE",
	        AV_PIX_FMT_YUV420P => "AV_PIX_FMT_YUV420P",
	        AV_PIX_FMT_YUYV422 => "AV_PIX_FMT_YUYV422",
	        AV_PIX_FMT_YUV422P => "AV_PIX_FMT_YUV422P",
	        AV_PIX_FMT_YUV444P => "AV_PIX_FMT_YUV444P",
	        AV_PIX_FMT_YUV410P => "AV_PIX_FMT_YUV410P",
	        AV_PIX_FMT_YUV411P => "AV_PIX_FMT_YUV411P",
            _                  => "AV_PIX_FMT_NONE",
        }
    }
}

impl From<AVPixelFormat> for PixelFormat {
    fn from(pix: AVPixelFormat) -> PixelFormat {
        PixelFormat(pix)
    }
}

impl Deref for PixelFormat {
    type Target = AVPixelFormat;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Serialize for PixelFormat {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        serializer.serialize_unit_variant("AVPixelFormat", self.0 as u32, self.to_static_str())
    }
}

struct PixelFormatVisitor;

impl<'de> Visitor<'de> for PixelFormatVisitor {
    type Value = PixelFormat;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a variant of AVPixelFormat")
    }

    fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error> where A: EnumAccess<'de> {
        data.variant().map(|x| x.0)
    } 
}

impl<'de> Deserialize<'de> for PixelFormat {

    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        deserializer.deserialize_unit(PixelFormatVisitor)
    }

}