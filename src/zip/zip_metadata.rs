use std::io::{Error, SeekFrom};
use std::io::prelude::*;
use std::io::BufReader;
use super::eof_central_dir::*;
use super::central_dir_file_header::CentralDirectoryFileHeader;
use std::path::Path;
use std::fs::File;

pub struct ZipMetadataParser;
type CentralDirectories = Vec<CentralDirectoryFileHeader>;

impl ZipMetadataParser {

    pub fn parse_eof_central_dir<P>(file_path: P) -> Result<EndOfCentralDirectory, Error>
    where P: AsRef<Path> {
        let file = File::open(file_path)?;
        let mut buf_reader = BufReader::new(file);

        let mut buffer = vec![0; MIN_EOF_CENTRAL_DIRECTORY_SIZE];

        buf_reader.seek(SeekFrom::End(-1 * (MIN_EOF_CENTRAL_DIRECTORY_SIZE as i64)))?;
        buf_reader.read_exact(&mut buffer)?;

        Ok(EndOfCentralDirectory::from_bytes(&buffer))
    }
    
    pub fn parse_central_dir_headers<P>(file_path: P, start_offset: u32, size: u16) -> Result<CentralDirectories, Error>
    where P: AsRef<Path> {
        let file = File::open(file_path)?;
        let mut buf_reader = BufReader::new(file);

        buf_reader.seek(SeekFrom::Start(start_offset as u64))?;
        
        let mut central_directories: CentralDirectories = Vec::with_capacity(size as usize);
        
        for _ in 0..size as usize {
            central_directories.push(match CentralDirectoryFileHeader::from_reader(&mut buf_reader) {
                Ok(central_directory) => central_directory,
                Err(err) => return Err(err)
            });
        }

        Ok(central_directories)
    }


}