use std::marker::{Send, Sync};
use std::fmt;
use std::convert::{From};
use std::ops::{Deref};
use serde::{Serialize, Deserialize, Serializer, Deserializer};
use serde::de::{Visitor, EnumAccess};

use ffmpeg_sys::*;

#[derive(Debug)]
pub struct CodecId(AVCodecID);

unsafe impl Send for CodecId {}
unsafe impl Sync for CodecId {}

impl CodecId {
    fn to_static_str(&self) -> &'static str {
        match self.0 {
            AV_CODEC_ID_NONE     => "AV_CODEC_ID_NONE",
            AV_CODEC_ID_MJPEG    => "AV_CODEC_ID_MJPEG",
            AV_CODEC_ID_MPEG4    => "AV_CODEC_ID_MPEG4",
            AV_CODEC_ID_RAWVIDEO => "AV_CODEC_ID_RAWVIDEO",
            AV_CODEC_ID_JPEG2000 => "AV_CODEC_ID_JPEG2000",
            AV_CODEC_ID_H264     => "AV_CODEC_ID_H264",
            _                    => "AV_CODEC_ID_NONE",
        }
    }
}

impl From<AVCodecID> for CodecId {
    fn from(id: AVCodecID) -> CodecId {
        CodecId(id)
    }
}

impl Deref for CodecId {
    type Target = AVCodecID;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Serialize for CodecId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        serializer.serialize_unit_variant("AVPixelFormat", self.0 as u32, self.to_static_str())
    }
}

struct CodecIdVisitor;

impl<'de> Visitor<'de> for CodecIdVisitor {
    type Value = CodecId;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a variant of AVCodecID")
    }

    fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error> where A: EnumAccess<'de> {
        data.variant().map(|x| x.0)
    } 
}

impl<'de> Deserialize<'de> for CodecId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        deserializer.deserialize_unit(CodecIdVisitor)
    }
}