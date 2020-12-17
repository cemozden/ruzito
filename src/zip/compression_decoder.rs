use std::io::{Error, ErrorKind, Read, Write};

use super::mem_map::CompressionMethod;
use inflate::DeflateDecoder;

pub struct CompressionDecoder;

impl CompressionDecoder {
    pub fn decode_to_file<R,W>(compression_method: &CompressionMethod, reader: &mut R, writer: &mut W) -> std::io::Result<u64> where R: Read, W: Write {
        match compression_method {
            CompressionMethod::NoCompression => return std::io::copy(reader, writer),
            CompressionMethod::Deflate => {
                let mut deflate_decoder = DeflateDecoder::new(reader);
                std::io::copy(&mut deflate_decoder, writer)
            }
            _ => Err(Error::new(ErrorKind::InvalidInput, "Unknown Compression Method"))
        }
    }
}