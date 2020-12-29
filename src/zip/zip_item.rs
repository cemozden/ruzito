use std::{fs::File, io::{BufReader, BufWriter, Read, Seek, SeekFrom}, path::Path};

use super::{ExtractError, compression_decoder, date_time::ZipDateTime, local_file_header, mem_map::CompressionMethod};

#[derive(Debug)]
pub struct ZipItem {
    item_path: String,
    is_file: bool,
    uncompressed_size: u32,
    compressed_size: u32,
    compression_method: CompressionMethod,
    modified_date_time: ZipDateTime,
    start_offset: u32

}
impl ZipItem {

    pub fn new(compression_method: CompressionMethod, item_path: String, uncompressed_size: u32, compressed_size: u32, modified_date_time: ZipDateTime, start_offset: u32) -> Self {
        let is_file = !item_path.ends_with("/"); 
        Self {
            compression_method,
            item_path,
            is_file,
            uncompressed_size, 
            compressed_size,
            modified_date_time,
            start_offset
        }
    }

    pub fn item_path(&self) -> &String {
        &self.item_path
    }

    pub fn is_file(&self) -> bool {
        self.is_file
    }

    pub fn uncompressed_size(&self) -> u32 {
        self.uncompressed_size
    }

    pub fn compression_method(&self) -> &CompressionMethod {
        &self.compression_method
    }

    pub fn modified_date_time(&self) -> &ZipDateTime {
        &self.modified_date_time
    }


    pub fn extract<P>(&self, dest_path: P) -> Result<Box<dyn AsRef<Path>>, ExtractError> where P: AsRef<Path> {

        match dest_path.as_ref().parent() {
            Some(parent_path) => {
                let item_path = Some(&self.item_path)
                    .filter(|_| cfg!(windows))
                    .map(|p| p.replace("/", r"\"))
                    .unwrap_or(String::from(&self.item_path));

                let item_extract_dest_path = Path::new(parent_path).join(item_path);
                println!("{}", item_extract_dest_path.display());

                if !self.is_file() {
                    match std::fs::create_dir_all(item_extract_dest_path.clone()) {
                        Ok(_) =>  return Ok(Box::new(item_extract_dest_path)),
                        Err(_) => return Err(ExtractError::CreateDirError(format!("{}", &item_extract_dest_path.display())))
                    }
                } else {
                    let zip_file = File::open(&dest_path).map_err(|err| ExtractError::IOError(err))?;

                    // Check if parent folder is created
                    let output_file_parent_path = Path::new(&item_extract_dest_path).parent();

                    match output_file_parent_path {
                        Some(path) => {
                            if !path.exists() {
                                if let Err(_) = std::fs::create_dir_all(path) {
                                    return Err(ExtractError::CreateDirError(format!("{}", &item_extract_dest_path.display())))
                                }
                            }
                        },
                        None => return Err(ExtractError::CreateDirError(format!("{}", &item_extract_dest_path.display())))
                    }

                    let output_file = File::create(item_extract_dest_path.clone()).map_err(|_| ExtractError::FileCreationFailed)?;

                    let mut zip_file_reader = BufReader::new(zip_file);
                    let mut buf_writer = BufWriter::new(output_file);

                    let file_start_offset = self.start_offset();
                    zip_file_reader.seek(SeekFrom::Start(file_start_offset as u64)).map_err(|_| ExtractError::UnableToSeekZipItem(file_start_offset))?;

                    let local_file_header = local_file_header::LocalFileHeader::from_reader(&mut zip_file_reader).map_err(|err| ExtractError::IOError(err))?;
                    let content_start_offset = local_file_header.content_start_offset();
                    
                    zip_file_reader.seek(SeekFrom::Start(content_start_offset)).map_err(|_| ExtractError::UnableToSeekZipItem(file_start_offset))?;
                    
                    let mut content_reader = zip_file_reader.take(self.compressed_size() as u64);
                    compression_decoder::CompressionDecoder::decode_to_file(local_file_header.compression_method(), 
                            &mut content_reader, 
                            &mut buf_writer)
                                .map_err(|err| ExtractError::IOError(err))?;

                    Ok(Box::new(item_extract_dest_path))
                }
            },
            None => return Err(ExtractError::InvalidParentPath(format!("{}", dest_path.as_ref().display())))
        }

    }


    pub fn compressed_size(&self) -> u32 {
        self.compressed_size
    }

    pub fn start_offset(&self) -> u32 {
        self.start_offset
    }
}