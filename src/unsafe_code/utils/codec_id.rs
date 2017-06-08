use std::marker::{Send, Sync};
use std::fmt;
use std::convert::{From};
use std::ops::{Deref};
use serde::{Serialize, Deserialize, Serializer, Deserializer};
use serde::de::{Visitor, EnumAccess, Error};

use ffmpeg_sys::*;

#[derive(Debug, Clone, Copy, PartialEq)]
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

    fn from_str(item: &str) -> CodecId {
        match item {
            "AV_CODEC_ID_NONE"     => CodecId::from(AV_CODEC_ID_NONE),
            "AV_CODEC_ID_MJPEG"    => CodecId::from(AV_CODEC_ID_MJPEG),
            "AV_CODEC_ID_MPEG4"    => CodecId::from(AV_CODEC_ID_MPEG4),
            "AV_CODEC_ID_RAWVIDEO" => CodecId::from(AV_CODEC_ID_RAWVIDEO),
            "AV_CODEC_ID_JPEG2000" => CodecId::from(AV_CODEC_ID_JPEG2000),
            "AV_CODEC_ID_H264"      => CodecId::from(AV_CODEC_ID_H264),
            _                      => CodecId::from(AV_CODEC_ID_NONE),
        }
    }
}

impl From<AVCodecID> for CodecId {
    fn from(id: AVCodecID) -> CodecId {
        CodecId(id)
    }
}

impl Into<AVCodecID> for CodecId {
    fn into(self) -> AVCodecID {
        println!("{:?}", self.0);
        self.0
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
        serializer.serialize_unit_variant("AVCodecID", self.0 as u32, self.to_static_str())
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

    fn visit_str<A>(self, item: &str) -> Result<Self::Value, A> where A: Error {
        Ok(CodecId::from_str(item))
    }
}

impl<'de> Deserialize<'de> for CodecId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        deserializer.deserialize_unit(CodecIdVisitor)
    }
}