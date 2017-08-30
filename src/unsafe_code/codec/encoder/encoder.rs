use std::marker::{Send};
use std::convert::{From, AsRef, AsMut};
use std::ops::{Deref, DerefMut};

use unsafe_code::codec::{CodecContext, Codec};
use unsafe_code::{AsRawPtr, Packet, Frame, UnsafeError, UnsafeErrorKind};

use ffmpeg_sys::*;

pub struct EncodingCodec(Codec);

unsafe impl Send for EncodingCodec {}

impl AsRef<AVCodec> for EncodingCodec {
    fn as_ref(&self) -> &AVCodec {
        self.0.as_ref()
    }
}

impl AsMut<AVCodec> for EncodingCodec {
    fn as_mut(&mut self) -> &mut AVCodec {
        self.0.as_mut()
    }
}

impl From<Codec> for EncodingCodec {
    fn from(codec: Codec) -> EncodingCodec {
        unsafe {
            EncodingCodec(codec)
        }
    }
}

impl AsRawPtr<AVCodec> for EncodingCodec {
    fn as_ptr(&self) -> *const AVCodec {
        self.0.as_ptr()
    }

    fn as_mut_ptr(&mut self) -> *mut AVCodec {
        self.0.as_mut_ptr()
    } 
}

pub struct EncodingCodecContext(CodecContext, EncodingCodec);

impl EncodingCodecContext {
    pub fn new(codec: EncodingCodec, context: CodecContext) -> EncodingCodecContext {
        EncodingCodecContext(context, codec)
    }

    unsafe fn encode_raw_frame(&mut self, mut frame: Frame) -> Result<Vec<Packet>, UnsafeError> {    
        let ret = avcodec_send_frame(self.as_mut_ptr(), frame.as_mut_ptr());
        let mut vec = Vec::new();

        if ret < 0 {
            return Err(UnsafeError::new(UnsafeErrorKind::SendFrame(ret)));
        }

        while ret >= 0 {
            let packet = av_packet_alloc();
            let ret = avcodec_receive_packet(self.as_mut_ptr(), packet);

            if ret == -11 || ret == AVERROR_EOF {
                return Ok(vec);
            } else if ret < 0 {
                return Err(UnsafeError::new(UnsafeErrorKind::ReceivePacket(ret)));
            }

            vec.push(Packet::from(Box::from_raw(packet)));

        }

        Ok(vec)

    }

    pub fn encode_frame(&mut self, mut frame: Frame) -> Result<Vec<Packet>, UnsafeError> {
        unsafe {
            self.encode_raw_frame(frame)
        }
}

    pub fn encode_null_frame(&mut self) -> Result<Vec<Packet>, UnsafeError> {
        unsafe {
            self.encode_raw_frame(Frame::null())
        }
    }
}

impl AsRef<AVCodecContext> for EncodingCodecContext {
    fn as_ref(&self) -> &AVCodecContext {
        self.0.as_ref()
    }
}

impl AsMut<AVCodecContext> for EncodingCodecContext {
    fn as_mut(&mut self) -> &mut AVCodecContext {
        self.0.as_mut()
    }
}

impl AsRawPtr<AVCodecContext> for EncodingCodecContext {
    fn as_ptr(&self) -> *const AVCodecContext {
        self.0.as_ptr()
    }

    fn as_mut_ptr(&mut self) -> *mut AVCodecContext {
        self.0.as_mut_ptr()
    }
}

