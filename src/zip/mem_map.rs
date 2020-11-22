const FILE_HEADER_SIGNATURE: u32 = 0x504b0304;

pub struct LocalFile {
    header: LocalFileHeader
}

pub struct LocalFileHeader {
    signature: u32,
    version: u16,
    general_purpose_flag: u16

}