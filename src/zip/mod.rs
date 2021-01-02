use std::{ffi::OsString, io::{Error, Write}, path::Path, process::exit};

use self::{encryption::ZipCryptoError, mem_map::EncryptionMethod, zip_item::ZipItem};


mod local_file_header;
mod eof_central_dir;
mod central_dir_file_header;
mod zip_metadata;
mod date_time;
mod compression_decoder;
mod encryption;

pub mod mem_map;
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
    UnableToSeekZipItem(u32),
    IOError(std::io::Error),
    ZipCryptoError(ZipCryptoError)
}

#[derive(Debug)]
pub struct ZipFile {
    file_count: u16,
    zip_items: Vec<zip_item::ZipItem>,
    zip_file_path: OsString,
    file_encryption_method: EncryptionMethod
}

#[inline]
pub fn read_pass() -> Result<String, Error> {
    print!("Enter password: ");
    if let Err(err) = std::io::stdout().flush() {
        return Err(err)
    }
    let pass = match rpassword::read_password() {
        Ok(pass) => pass,
        Err(err) => return Err(err)
    };

    Ok(pass)
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
        
        let zip_items : Vec<ZipItem> = file_headers.into_iter()
            .map(|item| item.into())
            .collect();

        let file_encryption_method = if zip_items.len() > 0 {
            zip_items[0].encryption_method()
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

    pub fn extract_all(&mut self) {
        let mut item_iterator = self.zip_items.iter_mut();

        let zip_password = match self.file_encryption_method {
            EncryptionMethod::NoEncryption => None,
            _ => match read_pass() {
                Ok(pass) => Some(pass),
                Err(_) => None
            }
        };

        while let Some(item) = item_iterator.next() {
            let item_extract_result = item.extract(&zip_password, &self.zip_file_path);

            if let Err(err) = item_extract_result {
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