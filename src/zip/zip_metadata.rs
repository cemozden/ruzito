use std::io::{BufReader, Error, SeekFrom};
use std::io::prelude::*;

use super::eof_central_dir::{MIN_EOF_CENTRAL_DIRECTORY_SIZE, EndOfCentralDirectory};
use super::central_dir_file_header::CentralDirectoryFileHeader;
use std::path::Path;
use std::fs::File;

#[derive(Debug)]
pub struct ZipMetadata;

impl ZipMetadata {

    pub fn parse<P>(file_path: P) -> Result<(EndOfCentralDirectory, Vec<CentralDirectoryFileHeader>), Error> where P: AsRef<Path> {
        let mut file = File::open(file_path)?;
        let end_of_central_directory = ZipMetadata::parse_eof_central_dir(&mut file)?;
        let central_directory_file_headers = ZipMetadata::parse_central_dir_headers(file, &end_of_central_directory)?;

        Ok((end_of_central_directory, central_directory_file_headers))
    }

    fn parse_eof_central_dir(zip_file: &mut File) -> Result<EndOfCentralDirectory, Error> {
        let mut buffer = vec![0; MIN_EOF_CENTRAL_DIRECTORY_SIZE];

        zip_file.seek(SeekFrom::End(-1 * (MIN_EOF_CENTRAL_DIRECTORY_SIZE as i64)))?;
        zip_file.read_exact(&mut buffer)?;

        Ok(EndOfCentralDirectory::from(buffer.as_ref()))
    }
    
    fn parse_central_dir_headers(zip_file: File, eof_central_dir: &EndOfCentralDirectory) -> Result<Vec<CentralDirectoryFileHeader>, Error> {

        let mut buf_reader = BufReader::new(zip_file);
        let central_dir_count = eof_central_dir.total_num_of_central_dir() as usize;

        buf_reader.seek(SeekFrom::Start(eof_central_dir.start_offset() as u64))?;

        (0..central_dir_count).into_iter()
            .map(|_| CentralDirectoryFileHeader::from_reader(&mut buf_reader))
            .collect()
    }

}
