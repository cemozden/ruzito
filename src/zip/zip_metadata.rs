use std::io::{Error, SeekFrom};
use std::io::prelude::*;
use std::io::BufReader;
use std::io::Cursor;
use super::eof_central_dir::*;
use super::central_dir_file_header::CentralDirectoryFileHeader;
use super::mem_map::END_OF_CENTRAL_DIR_SIGNATURE;

pub struct ZipMetadataParser;

impl ZipMetadataParser {

    pub fn parse_eof_central_dir<R>(reader: R) -> Result<EndOfCentralDirectory, Error>
    where R: Read + Seek {
        let mut buf_reader = BufReader::new(reader);

        let mut buffer = vec![0; MIN_EOF_CENTRAL_DIRECTORY_SIZE];

        buf_reader.seek(SeekFrom::End(-1 * (MIN_EOF_CENTRAL_DIRECTORY_SIZE as i64)))?;
        buf_reader.read_exact(&mut buffer)?;

        println!("{:?}", buffer);
        Ok(EndOfCentralDirectory::from_bytes(&buffer))
    }
    
    pub fn parse_central_dir_headers<R>(reader: R, start_offset: u32) -> Vec<CentralDirectoryFileHeader> 
    where R: Read + Seek {
        //TODO: Implement this
        vec![]
    }


}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_values_are_as_expected() {
        let eof_central_bytes = Cursor::new([80, 75, 5, 6, 0, 0, 0, 0, 55, 0, 55, 0, 59, 22, 0, 0, 53, 174, 0, 0, 0, 0]);
        let eof_central_dir = ZipMetadataParser::parse_eof_central_dir(eof_central_bytes).unwrap();
        
        assert_eq!(eof_central_dir.signature(), END_OF_CENTRAL_DIR_SIGNATURE as u32);
        assert_eq!(eof_central_dir.num_of_disk(), 0);
        assert_eq!(eof_central_dir.num_of_disk_start_central_dir(), 0);
        assert_eq!(eof_central_dir.num_of_central_dir(), 55);
        assert_eq!(eof_central_dir.total_num_of_central_dir(), 55);
        assert_eq!(eof_central_dir.size_of_central_dir(), 5691);
        assert_eq!(eof_central_dir.start_offset(), 44597);
        assert_eq!(eof_central_dir.zip_comment_len(), 0);
        assert_eq!(eof_central_dir.zip_comment(), "");
    }

}