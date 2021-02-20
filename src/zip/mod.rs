use std::{ffi::OsString, path::Path, process::exit};


use self::{encryption::{zip_crypto::ZipCryptoError}, mem_map::EncryptionMethod, options::ExtractOptions, zip_item::ZipItem};


mod local_file_header;
mod eof_central_dir;
mod central_dir_file_header;
mod zip_metadata;
mod date_time;
mod compression_decoder;
mod encryption;
mod crc32;

pub mod options;
pub mod mem_map;
pub mod zip_item;

use zip::crc32::calculate_checksum;

#[derive(Debug)]
pub enum ZipError {
    FileIOError(std::io::Error)
}

#[derive(Debug)]
pub enum ExtractError {
    InvalidParentPath(String),
    CreateDirError(String),
    FileCreationFailed,
    UnableToSeekZipItem(u32),
    IOError(std::io::Error),
    ZipCryptoError(ZipCryptoError),
}

#[derive(Debug)]
pub struct ZipFile {
    file_count: u16,
    zip_items: Vec<zip_item::ZipItem>,
    zip_file_path: OsString,
    file_encryption_method: EncryptionMethod
}

pub struct ZipFileIntoIterator<'a> {
    index: usize,
    zip_files: &'a Vec<zip_item::ZipItem>
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
        
        let zip_items : Vec<ZipItem> = file_headers.into_iter()
            .map(|item| item.into())
            .collect();

        let file_encryption_method = if zip_items.len() > 0 {
            if let Some(zip_item) = zip_items.iter().find(|zip_item| zip_item.encryption_method() != EncryptionMethod::NoEncryption) {
                zip_item.encryption_method()
            }
            else {
                EncryptionMethod::NoEncryption
            }
        } else {
            EncryptionMethod::NoEncryption
        };

        Ok(ZipFile {
            zip_items, 
            file_count: eof_central_dir.total_num_of_central_dir(),
            zip_file_path: file_path_os_string,
            file_encryption_method
        })
    }

    pub fn extract_all(&mut self, options: ExtractOptions) {
        let mut item_iterator = self.zip_items.iter_mut();

        while let Some(item) = item_iterator.next() {
            let zip_file_path = &self.zip_file_path;
            let item_extract_result = item.extract(&options);
            let is_file = item.is_file();

            match item_extract_result {
                Ok(path) => {
                    let output_file_path = Path::new(zip_file_path).join(path.as_ref());
                    
                    if is_file {
                        match calculate_checksum(output_file_path) {
                            Ok(checksum) => {
                                if checksum != item.crc32() {
                                    eprintln!("CRC32 checksum do not match! Exiting...");
                                    exit(-1);
                                }
                            },
                            Err(err) => {
                                eprintln!("I/O error occured while decrypting the file! {}", err);
                                exit(-1);
                            }
                        }
                    }
                },
                Err(err) => {
                println!("An error occured while extracting the file {}!", item.item_path());
                match err {
                        ExtractError::InvalidParentPath(parent_path) => eprintln!("Invalid parent path to extract files! Given Path: {}", parent_path),
                        ExtractError::CreateDirError(dir_path) => eprintln!("Unable to create directory of {}", dir_path),
                        ExtractError::FileCreationFailed => eprintln!("Unable to create the extracted file!"),
                        ExtractError::UnableToSeekZipItem(offset) => eprintln!("Unable to seek the ZIP file!, Failed offset: {}", offset),
                        ExtractError::IOError(err) => eprintln!("I/O error occured while extracting the file! {}", err),
                        ExtractError::ZipCryptoError(err) => {
                            match err {
                                ZipCryptoError::InvalidPassword(_) => { 
                                    eprintln!("Incorrect Password. Exiting.."); 
                                    exit(-1);
                                },
                                ZipCryptoError::IOError(err) => eprintln!("I/O error occured while decrypting the file! {}", err)
                            }
                        }
                    }

                    break;
                }
            }

        }
    }

    pub fn file_count(&self) -> u16 {
        self.file_count
    }

    pub fn zip_file_path(&self) -> &OsString {
        &self.zip_file_path
    }

    pub fn file_encryption_method(&self) -> &EncryptionMethod {
        &self.file_encryption_method
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