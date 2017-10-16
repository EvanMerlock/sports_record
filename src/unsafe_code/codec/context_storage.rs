use std::marker::{Send};

use unsafe_code::codec::{EncodingCodecContext, DecodingCodecContext};
use unsafe_code::sws::SWSContext;


#[derive(Clone)]
pub struct CodecStorage {
    pub encoding_context: EncodingCodecContext,
    pub decoding_context: DecodingCodecContext,
    pub jpeg_context: EncodingCodecContext,
    pub jpeg_sws_context: SWSContext,
    pub sws_context: SWSContext,
}

impl CodecStorage {

    pub fn new(enc: EncodingCodecContext, dec: DecodingCodecContext, jpeg: EncodingCodecContext, sws: SWSContext, jpeg_sws: SWSContext) -> CodecStorage {
        CodecStorage {
            encoding_context: enc,
            jpeg_context: jpeg,
            decoding_context: dec,
            jpeg_sws_context: jpeg_sws,
            sws_context: sws,
        }
    }

}

unsafe impl Send for CodecStorage {}