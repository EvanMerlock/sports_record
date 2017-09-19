use std::marker::{Send};

use unsafe_code::codec::{EncodingCodecContext, DecodingCodecContext};
use unsafe_code::sws::SWSContext;


#[derive(Clone)]
pub struct CodecStorage {
    pub encoding_context: EncodingCodecContext,
    pub decoding_context: DecodingCodecContext,
    pub sws_context: SWSContext,
}

impl CodecStorage {

    pub fn new(enc: EncodingCodecContext, dec: DecodingCodecContext, sws: SWSContext) -> CodecStorage {
        CodecStorage {
            encoding_context: enc,
            decoding_context: dec,
            sws_context: sws,
        }
    }

}

unsafe impl Send for CodecStorage {}