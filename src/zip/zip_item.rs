use super::{date_time::ZipDateTime, mem_map::CompressionMethod};

#[derive(Debug)]
pub struct ZipItem {
    item_path: String,
    is_file: bool,
    uncompressed_size: u32,
    compressed_size: u32,
    compression_method: CompressionMethod,
    modified_date_time: ZipDateTime,
    start_offset: u32

}
impl ZipItem {

    pub fn new(compression_method: CompressionMethod, item_path: String, uncompressed_size: u32, compressed_size: u32, modified_date_time: ZipDateTime, start_offset: u32) -> Self {
        let is_file = !item_path.ends_with("/"); 
        Self {
            compression_method,
            item_path,
            is_file,
            uncompressed_size, 
            compressed_size,
            modified_date_time,
            start_offset
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

    pub fn start_offset(&self) -> u32 {
        self.start_offset
    }
}