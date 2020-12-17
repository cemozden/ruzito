use std::{ffi::OsString, fs::File, io::{BufReader, BufWriter, Read, Seek, SeekFrom}, path::Path};

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
        self.zip_items.iter().map(|item| self.extract_zip_item(item)).collect()
    }

    pub fn extract_zip_item(&self, zip_item: &zip_item::ZipItem) -> Result<ZipItemExtract, ExtractError> {
        let zip_file_path = Path::new(&self.zip_file_path);

        match zip_file_path.parent() {
            Some(parent_path) => {
                let item_extract_dest_path = Path::new(parent_path).join(zip_item.item_path());
                println!("{:?}", &item_extract_dest_path.as_os_str());

                if !zip_item.is_file() {
                    match std::fs::create_dir_all(item_extract_dest_path.clone()) {
                        Ok(_) =>  return Ok(ZipItemExtract {
                                                file_path: OsString::from(item_extract_dest_path),
                                                item_size: 0,
                                                crc32: 0
                                            }),
                        Err(_) => return Err(ExtractError::CreateDirError(format!("{:?}", &item_extract_dest_path)))
                    }
                } else {
                    let zip_file = File::open(&self.zip_file_path).map_err(|_| ExtractError::InvalidZipFile(format!("{:?}", &self.zip_file_path)))?;
                    let output_file = File::create(item_extract_dest_path.clone()).map_err(|_| ExtractError::FileCreationFailed)?;

                    let mut zip_file_reader = BufReader::new(zip_file);
                    let mut buf_writer = BufWriter::new(output_file);

                    let file_start_offset = zip_item.start_offset();
                    zip_file_reader.seek(SeekFrom::Start(file_start_offset as u64)).map_err(|_| ExtractError::UnableToSeekZipItem(file_start_offset))?;

                    let local_file_header = local_file_header::LocalFileHeader::from_reader(&mut zip_file_reader).map_err(|err| ExtractError::IOError(err))?;
                    let content_start_offset = local_file_header.content_start_offset();
                    
                    zip_file_reader.seek(SeekFrom::Start(content_start_offset)).map_err(|_| ExtractError::UnableToSeekZipItem(file_start_offset))?;
                    
                    let mut content_reader = zip_file_reader.take(zip_item.compressed_size() as u64);
                    let decode_to_file = compression_decoder::CompressionDecoder::decode_to_file(local_file_header.compression_method(), 
                            &mut content_reader, 
                            &mut buf_writer)
                                .map_err(|err| ExtractError::IOError(err))?;

                    Ok(ZipItemExtract {
                        file_path: OsString::from(item_extract_dest_path),
                        item_size: decode_to_file as u32,
                        crc32: local_file_header.crc32()
                    })
                }
            },
            None => return Err(ExtractError::InvalidParentPath(format!("{:?}", &self.zip_file_path)))
        }

    }

    //TODO: Write folder creator with inline

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