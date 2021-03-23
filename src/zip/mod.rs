use std::{ffi::OsString, fs::{File, OpenOptions}, io::{BufReader, Seek, SeekFrom, Write}, path::Path, process::exit};


use self::{encryption::{zip_crypto::ZipCryptoError}, local_file_header::LocalFileHeader, central_dir_file_header::CentralDirectoryFileHeader, eof_central_dir::EndOfCentralDirectory, mem_map::EncryptionMethod, options::{ExtractOptions, ZipOptions}, zip_item::ZipItem};


mod local_file_header;
mod eof_central_dir;
mod central_dir_file_header;
mod zip_metadata;
mod date_time;
mod compression_decoder;
mod compression_encoder;
mod encryption;
mod crc32;

pub mod options;
pub mod mem_map;
pub mod zip_item;
pub mod zip_item_creator;

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
pub enum ZipCreatorError {
    InvalidPath(OsString),
    IOError(std::io::Error),
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

    pub fn create(file_count: u16, zip_items: Vec<zip_item::ZipItem>, zip_file_path: OsString, file_encryption_method: EncryptionMethod) -> Self {
        Self {
               file_count,
               zip_items,
               zip_file_path,
               file_encryption_method
        }
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

    pub fn create_zip_file(&mut self, zip_options: &ZipOptions) -> Result<(), ZipError> {
        let mut dest_path_file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(zip_options.dest_path()).map_err(|err| ZipError::FileIOError(err))?;
        let mut cdfh_vec = Vec::with_capacity(self.zip_items.len());

        for zip_item in &mut self.zip_items {
            println!("{}", zip_item.item_path());

            let zip_item_start_offset = dest_path_file.seek(SeekFrom::End(0))
                       .map_err(|err| ZipError::FileIOError(err))?;

            zip_item.update_start_offset(zip_item_start_offset as u32);

            let reader = match ZipFile::generate_file_reader(zip_item, zip_options) {
                Ok(reader) => reader,
                Err(err) => {
                    return Err(err);
                }
            };

            if zip_item.is_file() {
                // Unwrap is safe here. We make sure that there'll always be a reader for each file.
                let mut buf_reader = reader.unwrap();

                let local_file_header = LocalFileHeader::from_zip_item(zip_item);
                //Write local file header
                dest_path_file.write_all(&local_file_header.to_binary())
                    .map_err(|err| ZipError::FileIOError(err))?;

                let file_start_offset = dest_path_file.seek(SeekFrom::Current(0))
                                .map_err(|err| ZipError::FileIOError(err))?;    

                compression_encoder::CompressionEncoder::encode_to_file(
                                &zip_item.compression_method(),
                                &mut buf_reader,
                         &mut dest_path_file)
                                 .map_err(|err| ZipError::FileIOError(err)
                                )?;

                let file_end_offset = dest_path_file.seek(SeekFrom::Current(0))
                                .map_err(|err| ZipError::FileIOError(err))?;    
                            
                let file_compressed_size = (file_end_offset - file_start_offset) as u32;
                zip_item.update_compressed_size(file_compressed_size);
                
            }

            dest_path_file.seek(SeekFrom::Start(zip_item.start_offset() as u64))
                .map_err(|err| ZipError::FileIOError(err))?;

            //Update local file header with updated compressed size
            dest_path_file.write_all(&LocalFileHeader::from_zip_item(zip_item).to_binary())
                .map_err(|err| ZipError::FileIOError(err))?;

            dest_path_file.seek(SeekFrom::End(0))
                .map_err(|err| ZipError::FileIOError(err))?;
            
            cdfh_vec.push(CentralDirectoryFileHeader::from_zip_item(zip_item));
        }

        let mut cdfh_size = 0;
        let cdfh_start_offset = dest_path_file.seek(SeekFrom::End(0))
            .map_err(|err| ZipError::FileIOError(err))? as u32;

        for cdfh in cdfh_vec {
            let cdfh_bin = cdfh.to_binary();
            cdfh_size = cdfh_size + cdfh_bin.len();
            dest_path_file.write_all(&&cdfh_bin)
                .map_err(|err| ZipError::FileIOError(err))?;
        }
                
        let item_count = self.zip_items.len() as u16;
        let eocd = EndOfCentralDirectory::from_zip_creator(item_count, cdfh_size as u32, cdfh_start_offset);
        let mut  eocd_bytes = eocd.to_binary(); 

        dest_path_file.write_all(&mut eocd_bytes).map_err(|err| ZipError::FileIOError(err))?;

        Ok(())
    }

    fn generate_file_reader(zip_item: &mut ZipItem, zip_options: &ZipOptions) -> Result<Option<BufReader<File>>, ZipError> {

        let file_path_on_disk = Path::new(zip_options.base_path()).join(zip_item.item_path());
        let zip_item_reader;

        if zip_item.is_file() {
            let file_to_zip = File::open(file_path_on_disk)
                .map_err(|err| ZipError::FileIOError(err))?;

            let buf_reader = BufReader::new(file_to_zip);
            zip_item_reader = Some(buf_reader);
        } 
        else {
            zip_item_reader = None;
        }

        Ok(zip_item_reader)
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