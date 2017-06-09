use std::marker::{Send};

use unsafe_code::CodecContext;

pub struct CodecStorage {
    pub encoding_context: CodecContext,
    pub decoding_context: CodecContext,
}

impl CodecStorage {

    pub fn new(enc: CodecContext, dec: CodecContext) -> CodecStorage {
        CodecStorage {
            encoding_context: enc,
            decoding_context: dec,
        }
    }

}

unsafe impl Send for CodecStorage {}