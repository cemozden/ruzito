use super::{date_time::ZipDateTime, mem_map::CompressionMethod};
use std::path::Path;
#[derive(Debug)]
pub struct ZipItem {
    item_path: String,
    is_file: bool,
    uncompressed_size: u32,
    compressed_size: u32,
    compression_method: CompressionMethod,
    modified_date_time: ZipDateTime

}

impl ZipItem {

    pub fn new(compression_method: CompressionMethod, item_path: String, uncompressed_size: u32, compressed_size: u32, modified_date_time: ZipDateTime) -> Self {
        let is_file = Path::new(&item_path).extension().is_some();
        Self {
            compression_method,
            item_path,
            is_file,
            uncompressed_size, 
            compressed_size,
            modified_date_time
        }
    }

    pub fn item_path(&self) -> &String {
        &self.item_path
    }

    pub fn is_file(&self) -> bool {
        self.is_file
    }

    pub fn uncompressed_size(&self) -> u32 {
        self.uncompressed_size
    }

    pub fn compression_method(&self) -> &CompressionMethod {
        &self.compression_method
    }

    pub fn extract_file(&self) -> &[u8] {
        todo!()
    }

    pub fn compressed_size(&self) -> u32 {
        self.compressed_size
    }
}