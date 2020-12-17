use super::mem_map::END_OF_CENTRAL_DIR_SIGNATURE;
use byteorder::{LittleEndian, ByteOrder};
use std::str;

pub const MIN_EOF_CENTRAL_DIRECTORY_SIZE: usize = 22;

#[derive(Debug)]
pub struct EndOfCentralDirectory {
    signature: u32,
    num_of_disk: u16,
    num_of_disk_start_central_dir: u16,
    num_of_central_dir: u16,
    total_num_of_central_dir: u16,
    size_of_central_dir: u32,
    start_offset: u32,
    zip_comment_len: u16,
    zip_comment: String
}

impl EndOfCentralDirectory {
    pub fn start_offset(&self) -> u32 {
        self.start_offset
    }

    pub fn total_num_of_central_dir(&self) -> u16 {
        self.total_num_of_central_dir
    }

}

impl From<&[u8]> for EndOfCentralDirectory {
    fn from(eof_bin: &[u8]) -> Self {
        assert_from_bytes(eof_bin);

        let zip_comment_len = LittleEndian::read_u16(&eof_bin[20..22]);
        let zip_comment_end_offset = MIN_EOF_CENTRAL_DIRECTORY_SIZE as u16 + zip_comment_len;

        EndOfCentralDirectory {
            signature: END_OF_CENTRAL_DIR_SIGNATURE,
            num_of_disk: LittleEndian::read_u16(&eof_bin[4..6]),
            num_of_disk_start_central_dir: LittleEndian::read_u16(&eof_bin[6..8]),
            num_of_central_dir: LittleEndian::read_u16(&eof_bin[8..10]),
            total_num_of_central_dir: LittleEndian::read_u16(&eof_bin[10..12]),
            size_of_central_dir: LittleEndian::read_u32(&eof_bin[12..16]),
            start_offset: LittleEndian::read_u32(&eof_bin[16..20]),
            zip_comment_len: LittleEndian::read_u16(&eof_bin[20..22]),
            zip_comment: String::from(str::from_utf8(&eof_bin[22..zip_comment_end_offset as usize]).unwrap())
        }
    }
}

fn assert_from_bytes(eof_bytes: &[u8]) {
    assert!(eof_bytes.len() >= MIN_EOF_CENTRAL_DIRECTORY_SIZE);
    let signature = LittleEndian::read_u32(&eof_bytes[0..4]);

    assert_eq!(signature, END_OF_CENTRAL_DIR_SIGNATURE);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic]
    fn test_eof_dir_assertions() {
        let bin = [1, 2, 3, 4];
        // Check if minimum size of eof central directory is greater or equal than 22
        EndOfCentralDirectory::from(bin.as_ref());

        let bin = [1, 2, 3, 4, 1, 2, 3, 4, 1, 2, 3, 4, 1, 2, 3, 4, 1, 2, 3, 4, 1, 2, 3, 4];
        // Check whether function panics if signature is missing.
        EndOfCentralDirectory::from(bin.as_ref());
    }

    #[test]
    fn eof_central_dir_parsed_as_expected() {
        let bin = [0x50, 0x4B, 0x05, 0x06, 0x00, 0x00, 0x00, 0x00, 0x09, 0x00, 0x09, 0x00, 0x13, 0x02, 0x00, 0x00, 0x77, 0x8B, 0x00, 0x00, 0x00, 0x00];

        let eof_central_dir = EndOfCentralDirectory::from(bin.as_ref());

        assert_eq!(eof_central_dir.signature, END_OF_CENTRAL_DIR_SIGNATURE as u32);
        assert_eq!(eof_central_dir.num_of_disk, 0);
        assert_eq!(eof_central_dir.num_of_disk_start_central_dir, 0);
        assert_eq!(eof_central_dir.num_of_central_dir, 9);
        assert_eq!(eof_central_dir.total_num_of_central_dir, 9);
        assert_eq!(eof_central_dir.size_of_central_dir, 531);
        assert_eq!(eof_central_dir.start_offset, 35703);
        assert_eq!(eof_central_dir.zip_comment_len, 0);
        assert_eq!(eof_central_dir.zip_comment, "");

    }

}