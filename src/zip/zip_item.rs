use std::{ffi::OsString, fs::File, io::{BufReader, BufWriter, Read, Seek, SeekFrom}, path::Path};

use super::{ExtractError, ZipItemExtract, compression_decoder, date_time::ZipDateTime, local_file_header, mem_map::CompressionMethod};

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


    pub fn extract<P>(&self, dest_path: P) -> Result<ZipItemExtract, ExtractError> where P: AsRef<Path> {

        match dest_path.as_ref().parent() {
            Some(parent_path) => {
                let item_extract_dest_path = Path::new(parent_path).join(self.item_path());
                println!("{:?}", &item_extract_dest_path.as_os_str());

                if !self.is_file() {
                    match std::fs::create_dir_all(item_extract_dest_path.clone()) {
                        Ok(_) =>  return Ok(ZipItemExtract {
                                                file_path: OsString::from(item_extract_dest_path),
                                                item_size: 0,
                                                crc32: 0
                                            }),
                        Err(_) => return Err(ExtractError::CreateDirError(format!("{:?}", &item_extract_dest_path)))
                    }
                } else {
                    let zip_file = File::open(&dest_path).map_err(|_| ExtractError::InvalidZipFile(format!("{:?}", &dest_path.as_ref().clone())))?;
                    let output_file = File::create(item_extract_dest_path.clone()).map_err(|_| ExtractError::FileCreationFailed)?;

                    let mut zip_file_reader = BufReader::new(zip_file);
                    let mut buf_writer = BufWriter::new(output_file);

                    let file_start_offset = self.start_offset();
                    zip_file_reader.seek(SeekFrom::Start(file_start_offset as u64)).map_err(|_| ExtractError::UnableToSeekZipItem(file_start_offset))?;

                    let local_file_header = local_file_header::LocalFileHeader::from_reader(&mut zip_file_reader).map_err(|err| ExtractError::IOError(err))?;
                    let content_start_offset = local_file_header.content_start_offset();
                    
                    zip_file_reader.seek(SeekFrom::Start(content_start_offset)).map_err(|_| ExtractError::UnableToSeekZipItem(file_start_offset))?;
                    
                    let mut content_reader = zip_file_reader.take(self.compressed_size() as u64);
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
            None => return Err(ExtractError::InvalidParentPath(format!("{:?}", dest_path.as_ref())))
        }

    }


    pub fn compressed_size(&self) -> u32 {
        self.compressed_size
    }

    pub fn start_offset(&self) -> u32 {
        self.start_offset
    }
}