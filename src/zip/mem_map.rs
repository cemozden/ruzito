pub const FILE_HEADER_SIGNATURE: u32 = 0x504b0304;
pub const DIGITAL_SIGNATURE_HEADER: u32 = 0x05054b50;
pub const END_OF_CENTRAL_DIR_SIGNATURE: u32 = 0x06054b50;
pub const CENTRAL_DIR_SIGNATURE: u32 = 0x02014b50;
pub const  ARCHIVE_EXTRA_DATA_SIGNATURE:u32 = 0x08064b50;

pub struct LocalFile {
    header: LocalFileHeader
}

pub struct LocalFileHeader {
    signature: u32,
    version: u16,
    general_purpose_flag: u16

}