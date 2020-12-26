use std::{ffi::OsString, path::Path};

mod mem_map;
mod local_file_header;
mod eof_central_dir;
mod central_dir_file_header;
mod archive_extra_data_record;
mod data_descriptor;
mod zip_metadata;
mod date_time;
mod compression_decoder;

pub mod zip_item;

#[derive(Debug)]
pub enum ZipError {
    FileIOError(std::io::Error)
}

#[derive(Debug)]
pub enum ExtractError {
    InvalidParentPath(String),
    CreateDirError(String),
    FileCreationFailed,
    InvalidZipFile(String),
    UnableToSeekZipItem(u32),
    IOError(std::io::Error),
    UnknownCompressionMethod
}

#[derive(Debug)]
pub struct ZipFile {
    file_count: u16,
    zip_items: Vec<zip_item::ZipItem>,
    zip_file_path: OsString
}

pub struct ZipFileIntoIterator<'a> {
    index: usize,
    zip_files: &'a Vec<zip_item::ZipItem>
}

#[derive(Debug)]
pub struct ZipItemExtract {
    file_path: OsString,
    item_size: u32,
    crc32: u32
}



impl<'a> ZipFileIntoIterator<'a> {
    pub fn new(zip_files: &'a Vec<zip_item::ZipItem>) -> Self {
        Self {
            index: 0,
            zip_files: zip_files
        }
    }
}
impl ZipFile {
    pub fn new<P>(zip_file_path: P) -> Result<Self, ZipError>
    where P: AsRef<Path> {
        let file_path_os_string = OsString::from(zip_file_path.as_ref().as_os_str());

        let (eof_central_dir, file_headers) = zip_metadata::ZipMetadata::parse(zip_file_path)
            .map_err(|io_error| ZipError::FileIOError(io_error))?;
        
        let zip_items = file_headers.into_iter()
            .map(|item| item.into())
            .collect();
            
        Ok(ZipFile {
            zip_items, 
            file_count: eof_central_dir.total_num_of_central_dir(),
            zip_file_path: file_path_os_string
        })
    }

    pub fn extract_all(&self) -> Vec<Result<ZipItemExtract, ExtractError>> {
        self.zip_items.iter().map(|item| item.extract(&self.zip_file_path)).collect()
    }

    pub fn file_count(&self) -> u16 {
        self.file_count
    }

    pub fn zip_file_path(&self) -> &OsString {
        &self.zip_file_path
    }

    pub fn iter<'a>(&'a self) -> ZipFileIntoIterator<'a> {
        ZipFileIntoIterator::new(&self.zip_items)
    }
}

impl<'a> Iterator for ZipFileIntoIterator<'a> {
    type Item = &'a zip_item::ZipItem;

    fn next(&mut self) -> Option<Self::Item> {
        let next_item = self.zip_files.get(self.index);
        self.index += 1;
        next_item
    }
}

impl<'a> IntoIterator for &'a ZipFile {
    type Item = &'a zip_item::ZipItem;
    type IntoIter = ZipFileIntoIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        ZipFileIntoIterator::new(&self.zip_items)
    }
}