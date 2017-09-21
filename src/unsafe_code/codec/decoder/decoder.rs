use std::marker::{Send};
use std::convert::{From, AsRef, AsMut};
use std::ops::{Deref, DerefMut};

use std::ptr;
use std::ffi::CString;

use unsafe_code::{AsRawPtr, UnsafeError, CodecId, UnsafeErrorKind, Frame, CodecParameters};
use unsafe_code::format::Stream;
use unsafe_code::codec::{CodecContext, Codec};

use libc;
use ffmpeg_sys::*;

#[derive(Clone)]
pub struct DecodingCodec(Codec);

unsafe impl Send for DecodingCodec {}

impl AsRef<AVCodec> for DecodingCodec {
    fn as_ref(&self) -> &AVCodec {
        self.0.as_ref()
    }
}

impl AsMut<AVCodec> for DecodingCodec {
    fn as_mut(&mut self) -> &mut AVCodec {
        self.0.as_mut()
    }
}

impl From<Codec> for DecodingCodec {
    fn from(codec: Codec) -> DecodingCodec {
        unsafe {
            DecodingCodec(codec)
        }
    }
}

impl AsRawPtr<AVCodec> for DecodingCodec {
    fn as_ptr(&self) -> *const AVCodec {
        self.0.as_ptr()
    }

    fn as_mut_ptr(&mut self) -> *mut AVCodec {
        self.0.as_mut_ptr()
    }
}

pub struct DecodingCodecContext(CodecContext, DecodingCodec);

impl DecodingCodecContext {
    pub fn new(codec: DecodingCodec, context: CodecContext) -> DecodingCodecContext {
        DecodingCodecContext(context, codec)
    }

    pub fn open(&mut self) -> Result<(), UnsafeError> {
        unsafe {
            let ret = avcodec_open2(self.0.as_mut_ptr(), self.1.as_ptr(), ptr::null_mut());
            if ret < 0 {
                return Err(UnsafeError::new(UnsafeErrorKind::OpenDecoder(ret)));
            }
            Ok(())
        }
    }

    unsafe fn allocate_decoding_codec_from_av_stream(stream_config: &mut Stream) -> Result<DecodingCodecContext, UnsafeError> {
        let decoding_codec = Codec::new_decoder(CodecId::from((*stream_config.codecpar).codec_id));
        let temp_context = CodecContext::new_codec_based_context(&decoding_codec);
        let mut decoding_context = DecodingCodecContext::new(decoding_codec, temp_context);


        let ret = avcodec_parameters_to_context(decoding_context.as_mut_ptr(), stream_config.codecpar);
        if ret < 0 {
            return Err(UnsafeError::new(UnsafeErrorKind::OpenDecoder(ret)));
        }

        decoding_context.0.load_parameters_from_codec_parameters(&CodecParameters::from(stream_config.codecpar)).map_err(|x| UnsafeError::new(UnsafeErrorKind::OpenDecoder(x)))?;

        {
            let internal_ref = <DecodingCodecContext as AsMut<AVCodecContext>>::as_mut(&mut decoding_context);
            if internal_ref.codec_id == AV_CODEC_ID_H264 {
                av_opt_set((internal_ref as *mut AVCodecContext) as *mut libc::c_void, CString::new("preset").unwrap().as_ptr(), CString::new("ultrafast").unwrap().as_ptr(), 0);
                av_opt_set((internal_ref as *mut AVCodecContext) as *mut libc::c_void, CString::new("crf").unwrap().as_ptr(), CString::new("28").unwrap().as_ptr(), 0);
            }
        }

        try!(decoding_context.open());

        Ok(decoding_context)
}

    pub fn create_decoding_context_from_av_stream(stream: &mut Stream) -> Result<DecodingCodecContext, UnsafeError> {
        unsafe {
            DecodingCodecContext::allocate_decoding_codec_from_av_stream(stream)
        }
    }

    unsafe fn decode_raw_packet(&mut self, packet: &AVPacket) -> Result<Frame, UnsafeError> {
        let frame = av_frame_alloc();
        let ret = avcodec_send_packet(self.as_mut_ptr(), packet);
        if ret != 0 {
            return Err(UnsafeError::new(UnsafeErrorKind::SendPacket(ret)));        
        }

        let ret = avcodec_receive_frame(self.as_mut_ptr(), frame);
        if ret != 0 {
            return Err(UnsafeError::new(UnsafeErrorKind::ReceiveFrame(ret)));
        }

        Ok(Frame::from(frame))
    }

    pub fn decode_packet(&mut self, packet: &AVPacket) -> Result<Frame, UnsafeError> {
        unsafe {
            self.decode_raw_packet(packet)
        }
    }
}

impl AsRef<AVCodecContext> for DecodingCodecContext {
    fn as_ref(&self) -> &AVCodecContext {
        self.0.as_ref()
    }
}

impl AsMut<AVCodecContext> for DecodingCodecContext {
    fn as_mut(&mut self) -> &mut AVCodecContext {
        self.0.as_mut()
    }
}

impl AsRawPtr<AVCodecContext> for DecodingCodecContext {
    fn as_ptr(&self) -> *const AVCodecContext {
        self.0.as_ptr()
    }

    fn as_mut_ptr(&mut self) -> *mut AVCodecContext {
        self.0.as_mut_ptr()
    }
}

impl Clone for DecodingCodecContext {
    fn clone(&self) -> Self {
        let mut cloned_codec = self.1.clone();
        let mut cloned_context = self.0.clone();
        unsafe {
            let ret = avcodec_open2(cloned_context.as_mut_ptr(), cloned_codec.as_mut_ptr(), ptr::null_mut());
            if ret < 0 {
                panic!("Cloning a DecodingContext failed: ERR CODE - {}", ret);
            }
        }
        DecodingCodecContext(cloned_context, cloned_codec)
    }
}