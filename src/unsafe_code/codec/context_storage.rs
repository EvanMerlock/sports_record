use std::marker::{Send};

use unsafe_code::codec::{EncodingCodecContext, DecodingCodecContext};
use unsafe_code::sws::SWSContext;


#[derive(Clone)]
pub struct CodecStorage {
    pub encoding_context: EncodingCodecContext,
    pub decoding_context: DecodingCodecContext,
    pub png_context: EncodingCodecContext,
    pub png_sws_context: SWSContext,
    pub sws_context: SWSContext,
}

impl CodecStorage {

    pub fn new(enc: EncodingCodecContext, dec: DecodingCodecContext, png: EncodingCodecContext, sws: SWSContext, png_sws: SWSContext) -> CodecStorage {
        CodecStorage {
            encoding_context: enc,
            png_context: png,
            decoding_context: dec,
            png_sws_context: png_sws,
            sws_context: sws,
        }
    }

}

unsafe impl Send for CodecStorage {}