use std::io::{BufRead, Error, ErrorKind, Write};

use flate2::{Compression, bufread::DeflateEncoder};

use super::mem_map::CompressionMethod;

pub struct CompressionEncoder;

impl CompressionEncoder {
    pub fn encode_to_file<R,W>(compression_method: &CompressionMethod, reader: &mut R, writer: &mut W) -> std::io::Result<u64> where R: BufRead, W: Write {
        match compression_method {
            CompressionMethod::NoCompression => return std::io::copy(reader, writer),
            CompressionMethod::Deflate => {
                let mut deflate_encoder = DeflateEncoder::new(reader, Compression::best());
                std::io::copy(&mut deflate_encoder, writer)
            },
            _ => Err(Error::new(ErrorKind::InvalidInput, "Unknown Compression Method"))
        }
    }
}