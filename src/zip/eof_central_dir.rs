pub struct EndOfCentralDirectory {
    signature: u32,
    num_of_disk: u16,
    num_of_disk_start_central_dir: u16,
    num_of_central_dir: u16,
    size_of_central_dir: u32,
    start_offset: u32,
    zip_comment_len: u16,
}