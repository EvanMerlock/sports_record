use std::marker::{Send};
use std::convert::{From, AsRef, AsMut};
use std::ops::{Deref, DerefMut};

use std::ptr;
use std::ffi::CString;

use unsafe_code::codec::{CodecContext, Codec};
use unsafe_code::{AsRawPtr, Packet, Frame, UnsafeError, UnsafeErrorKind, CodecId, Rational};

use libc;
use ffmpeg_sys::*;

#[derive(Clone)]
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

    pub fn open(&mut self) -> Result<(), UnsafeError> {
        unsafe {
            let ret = avcodec_open2(self.0.as_mut_ptr(), self.1.as_ptr(), ptr::null_mut());
            if ret < 0 {
                return Err(UnsafeError::new(UnsafeErrorKind::OpenEncoder(ret)));
            }
            Ok(())
        }
    }
    
    unsafe fn allocate_encoding_context(codec_type: CodecId, height: i32, width: i32, time_base: Rational, gop_size: i32, max_b_frames: i32) -> Result<EncodingCodecContext, UnsafeError> {
        let encoding_codec = Codec::new_encoder(codec_type);
        let mut temp_context = CodecContext::new_codec_based_context(&encoding_codec);
        let mut encoding_context = EncodingCodecContext::new(encoding_codec, temp_context);

        {
            let internal_ref = <EncodingCodecContext as AsMut<AVCodecContext>>::as_mut(&mut encoding_context);

            internal_ref.height = height;
            internal_ref.width = width;

            internal_ref.time_base = time_base.into();

            internal_ref.gop_size = gop_size;
            internal_ref.max_b_frames = max_b_frames;
            internal_ref.pix_fmt = AV_PIX_FMT_YUV420P;
        }

        if *codec_type == AV_CODEC_ID_H264 {
            av_opt_set(encoding_context.as_mut_ptr() as *mut libc::c_void, CString::new("preset").unwrap().as_ptr(), CString::new("ultrafast").unwrap().as_ptr(), 0);
            av_opt_set(encoding_context.as_mut_ptr() as *mut libc::c_void, CString::new("crf").unwrap().as_ptr(), CString::new("28").unwrap().as_ptr(), 0);
        }

        try!(encoding_context.open());

        Ok(encoding_context)

    }

    pub fn create_encoding_context(codec_type: CodecId, height: i32, width: i32, time_base: Rational, gop_size: i32, max_b_frames: i32) -> Result<EncodingCodecContext, UnsafeError> {
        unsafe {
            EncodingCodecContext::allocate_encoding_context(codec_type, height, width, time_base, gop_size, max_b_frames)
        }
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

impl AsRef<CodecContext> for EncodingCodecContext {
    fn as_ref(&self) -> &CodecContext {
        &self.0
    }
}

impl AsMut<CodecContext> for EncodingCodecContext {
    fn as_mut(&mut self) -> &mut CodecContext {
        &mut self.0
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

impl Clone for EncodingCodecContext {
    fn clone(&self) -> Self {
        let mut cloned_codec = self.1.clone();
        let mut cloned_context = self.0.clone();
        unsafe {
            let ret = avcodec_open2(cloned_context.as_mut_ptr(), cloned_codec.as_mut_ptr(), ptr::null_mut());
            if ret < 0 {
                panic!("Cloning a DecodingContext failed: ERR CODE - {}", ret);
            }
        }
        EncodingCodecContext(cloned_context, cloned_codec)
    }
}