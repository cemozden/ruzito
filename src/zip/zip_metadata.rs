use std::io::{Error, SeekFrom};
use std::io::prelude::*;
use std::io::BufReader;
use super::eof_central_dir::*;
use super::central_dir_file_header::CentralDirectoryFileHeader;

pub struct ZipMetadataParser;

type CentralDirectories = Vec<CentralDirectoryFileHeader>;

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
    
    pub fn parse_central_dir_headers<R>(reader: &mut R, start_offset: u64, size: u32) -> Result<CentralDirectories, Error>
    where R: Read + Seek {
        reader.seek(SeekFrom::Start(start_offset))?;
        let mut central_directories: CentralDirectories = Vec::with_capacity(size as usize);
        
        for _ in 0..size as usize {
            //TODO: Control Result enum handle error
            central_directories.push(CentralDirectoryFileHeader::from_reader(reader).unwrap());
        }

        Ok(central_directories)
    }


}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    use super::super::mem_map::END_OF_CENTRAL_DIR_SIGNATURE;

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