use std::slice::from_raw_parts;

use unsafe_code::{UnsafeError, UnsafeErrorKind, Codec, CodecId, CodecContext, Rational, Frame, EncodingCodecContext, AsRawPtr};

use ffmpeg_sys::*;

impl EncodingCodecContext {

    unsafe fn allocate_png_codec(height: i32, width: i32, time_base: Rational) -> Result<EncodingCodecContext, UnsafeError> {

        let codec_ptr = Codec::new_encoder(CodecId::from(AVCodecID::AV_CODEC_ID_PNG));
        let mut jpeg_context_ptr = CodecContext::new_codec_based_context(&codec_ptr);
        {
            let jpeg_context: &mut AVCodecContext = jpeg_context_ptr.as_mut();

            jpeg_context.height = height;
            jpeg_context.width = width;

            jpeg_context.time_base = time_base.into();

            jpeg_context.pix_fmt = AVPixelFormat::AV_PIX_FMT_RGB24;
        }

        let mut encode = EncodingCodecContext::new(codec_ptr, jpeg_context_ptr);

        encode.open()?;

        Ok(encode)

    }

    pub fn create_png_context(height: i32, width: i32, time_base: Rational) -> Result<EncodingCodecContext, UnsafeError> {
        unsafe {
            EncodingCodecContext::allocate_png_codec(height, width, time_base)
        }
    }

    unsafe fn create_png_frame(&mut self, frame: &Frame) -> Result<Vec<u8>, UnsafeError> {
        let ret = avcodec_send_frame(self.as_mut_ptr(), frame.as_ptr());

        if ret < 0 {
            return Err(UnsafeError::new(UnsafeErrorKind::SendFrame(ret)));
        }

        let packet = av_packet_alloc();
        let ret = avcodec_receive_packet(self.as_mut_ptr(), packet);
        if ret < 0 {
            return Err(UnsafeError::new(UnsafeErrorKind::ReceivePacket(ret)));
        }

        let img_vec = {
            from_raw_parts((*packet).data, (*packet).size as usize).to_vec().clone()
        };
        Ok(img_vec)
    }

    pub fn encode_png_frame(&mut self, frame: &Frame) -> Result<Vec<u8>, UnsafeError> {
        unsafe {
            self.create_png_frame(frame)
        }
    }
}