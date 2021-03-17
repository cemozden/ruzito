use super::mem_map::END_OF_CENTRAL_DIR_SIGNATURE;
use byteorder::{LittleEndian, ByteOrder};
use std::{io::{Error, ErrorKind}, str};

pub const MIN_EOF_CENTRAL_DIRECTORY_SIZE: usize = 22;

#[derive(Debug)]
pub struct EndOfCentralDirectory {
    signature: u32,
    num_of_disk: u16,
    num_of_disk_start_central_dir: u16,
    num_of_central_dir: u16,
    total_num_of_central_dir: u16,
    size_of_central_dir: u32,
    cdfh_start_offset: u32,
    zip_comment_len: u16,
    zip_comment: String
}

impl EndOfCentralDirectory {
    pub fn cdfh_start_offset(&self) -> u32 {
        self.cdfh_start_offset
    }

    pub fn total_num_of_central_dir(&self) -> u16 {
        self.total_num_of_central_dir
    }

    pub fn from(eof_bin: &[u8]) -> Result<Self, Error> {

        if eof_bin.len() < MIN_EOF_CENTRAL_DIRECTORY_SIZE {
            return Err(Error::new(ErrorKind::InvalidData, format!("Invalid End of central directory signature! Bytes size: {:#}", eof_bin.len())));
        }
        let signature = LittleEndian::read_u32(&eof_bin[0..4]);

        if signature != END_OF_CENTRAL_DIR_SIGNATURE {
            return Err(Error::new(ErrorKind::InvalidData, format!("Invalid End of central directory signature! 4 bytes given: {:#}", signature)));
        }

        let zip_comment_len = LittleEndian::read_u16(&eof_bin[20..22]);
        let zip_comment_end_offset = MIN_EOF_CENTRAL_DIRECTORY_SIZE as u16 + zip_comment_len;

        Ok(EndOfCentralDirectory {
            signature: END_OF_CENTRAL_DIR_SIGNATURE,
            num_of_disk: LittleEndian::read_u16(&eof_bin[4..6]),
            num_of_disk_start_central_dir: LittleEndian::read_u16(&eof_bin[6..8]),
            num_of_central_dir: LittleEndian::read_u16(&eof_bin[8..10]),
            total_num_of_central_dir: LittleEndian::read_u16(&eof_bin[10..12]),
            size_of_central_dir: LittleEndian::read_u32(&eof_bin[12..16]),
            cdfh_start_offset: LittleEndian::read_u32(&eof_bin[16..20]),
            zip_comment_len: LittleEndian::read_u16(&eof_bin[20..22]),
            zip_comment: String::from(str::from_utf8(&eof_bin[22..zip_comment_end_offset as usize]).unwrap())
        })
    }

    pub fn from_zip_creator(num_of_cdfh: u16, cdfh_size: u32, cdfh_start_offset: u32) -> Self {
        Self {
           signature: END_OF_CENTRAL_DIR_SIGNATURE,
           num_of_disk: 0,
           num_of_disk_start_central_dir: 0,
           num_of_central_dir: num_of_cdfh,
           total_num_of_central_dir: num_of_cdfh,
           size_of_central_dir: cdfh_size,
           cdfh_start_offset,
           zip_comment_len: 0,
           zip_comment: String::from("")
        }
    }

    pub fn to_binary(self) -> Vec<u8> {
        let mut eof_bin = Vec::with_capacity(MIN_EOF_CENTRAL_DIRECTORY_SIZE);

        let mut signature_bytes = vec![0u8; 4];

        LittleEndian::write_u32(&mut signature_bytes, END_OF_CENTRAL_DIR_SIGNATURE);

        eof_bin.append(&mut signature_bytes);

        eof_bin
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic]
    fn test_eof_dir_assertions() {
        let bin = [1, 2, 3, 4];
        // Check if minimum size of eof central directory is greater or equal than 22
        EndOfCentralDirectory::from(bin.as_ref()).unwrap();

        let bin = [1, 2, 3, 4, 1, 2, 3, 4, 1, 2, 3, 4, 1, 2, 3, 4, 1, 2, 3, 4, 1, 2, 3, 4];
        // Check whether function panics if signature is missing.
        EndOfCentralDirectory::from(bin.as_ref()).unwrap();
    }

    #[test]
    fn eof_central_dir_parsed_as_expected() {
        let bin = [0x50, 0x4B, 0x05, 0x06, 0x00, 0x00, 0x00, 0x00, 0x09, 0x00, 0x09, 0x00, 0x13, 0x02, 0x00, 0x00, 0x77, 0x8B, 0x00, 0x00, 0x00, 0x00];

        let eof_central_dir = EndOfCentralDirectory::from(bin.as_ref()).unwrap();

        assert_eq!(eof_central_dir.signature, END_OF_CENTRAL_DIR_SIGNATURE as u32);
        assert_eq!(eof_central_dir.num_of_disk, 0);
        assert_eq!(eof_central_dir.num_of_disk_start_central_dir, 0);
        assert_eq!(eof_central_dir.num_of_central_dir, 9);
        assert_eq!(eof_central_dir.total_num_of_central_dir, 9);
        assert_eq!(eof_central_dir.size_of_central_dir, 531);
        assert_eq!(eof_central_dir.cdfh_start_offset, 35703);
        assert_eq!(eof_central_dir.zip_comment_len, 0);
        assert_eq!(eof_central_dir.zip_comment, "");

    }

}