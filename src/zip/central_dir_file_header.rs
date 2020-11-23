pub struct CentralDirectoryFileHeader {
    signature: u32,
    version: u16,
    version_needed_to_extract: u16,
    general_purpose_flag: u16,
    compression_method: u16,
    last_modified_file_time: u16,
    last_modified_file_date: u16,
    crc32: u32,
    compressed_size: u32,
    uncompressed_size: u32,
    file_name_length: u16,
    extra_field_length: u16,
    file_comment_length: u16,
    disk_number_start: u16,
    internal_file_attr: u16,
    external_file_attr: u16,
    relative_offset: u32
}