pub struct LocalFileHeader {
    signature: u32,
    version: u16,
    general_purpose_flag: u16,
    compression_method: u16,
    last_modifed_file_time: u16,
    last_modified_file_date: u16,
    crc32: u32,
    compressed_size: u32,
    uncompressed_size: u32,
    file_name_length: u16,
    file_name: String
}