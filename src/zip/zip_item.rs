use std::{fs::{File, OpenOptions}, io::{BufReader, BufWriter, Read, Seek, SeekFrom, Write}, path::Path};

use super::{ExtractError, ZipError, central_dir_file_header::CentralDirectoryFileHeader, compression_decoder, compression_encoder, date_time::ZipDateTime, encryption::zip_crypto::{ZipCryptoReader, ZipCryptoError}, local_file_header, mem_map::{CompressionMethod, EncryptionMethod}, options::{ExtractOptions, ZipOptions}};

#[derive(Debug)]
pub struct ZipItem {
    item_path: String,
    is_file: bool,
    uncompressed_size: u32,
    compressed_size: u32,
    compression_method: CompressionMethod,
    modified_date_time: ZipDateTime,
    start_offset: u32,
    encryption_method: EncryptionMethod,
    crc32: u32
}
impl ZipItem {

    pub fn new(compression_method: CompressionMethod, item_path: String, uncompressed_size: u32, compressed_size: u32, modified_date_time: ZipDateTime, start_offset: u32, encryption_method: EncryptionMethod, crc32: u32) -> Self {
        let is_file = !item_path.ends_with("/"); 
        Self {
            compression_method,
            item_path,
            is_file,
            uncompressed_size, 
            compressed_size,
            modified_date_time,
            start_offset,
            encryption_method,
            crc32
        }
    }

    pub fn item_path(&self) -> &String {
        &self.item_path
    }

    pub fn is_file(&self) -> bool {
        self.is_file
    }

    pub fn crc32(&self) -> u32 {
        self.crc32
    }

    pub fn uncompressed_size(&self) -> u32 {
        self.uncompressed_size
    }

    pub fn compression_method(&self) -> CompressionMethod {
        self.compression_method
    }

    pub fn modified_date_time(&self) -> &ZipDateTime {
        &self.modified_date_time
    }

    pub fn encryption_method(&self) -> EncryptionMethod {
        self.encryption_method
    }

    pub fn extract(&self, options: &ExtractOptions) -> Result<Box<dyn AsRef<Path>>, ExtractError> {

        let dest_path = Path::new(options.destination_path());

        let dest_path = if dest_path.is_dir() { dest_path } 
            else if dest_path.is_file() { dest_path.parent().unwrap() } // Unwrap is safe here. We check whether file exist at first.
            else { return Err(ExtractError::InvalidParentPath(format!("{}", dest_path.display()))) };

        let item_path = Some(&self.item_path)
            .filter(|_| cfg!(windows))
            .map(|p| p.replace("/", r"\"))
            .unwrap_or(String::from(&self.item_path));
        let item_extract_dest_path = Path::new(dest_path).join(item_path);

        if options.verbose_mode() {
            println!("{}", item_extract_dest_path.display());
        }

        if !self.is_file() {
            match std::fs::create_dir_all(item_extract_dest_path.clone()) {
                Ok(_) =>  return Ok(Box::new(item_extract_dest_path)),
                Err(_) => return Err(ExtractError::CreateDirError(format!("{}", &item_extract_dest_path.display())))
            }
        } else {
            let zip_file = File::open(options.zip_file_path()).map_err(|err| ExtractError::IOError(err))?;
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
            let file_size = if local_file_header.compression_method() == CompressionMethod::NoCompression 
                && local_file_header.encryption_method() != &EncryptionMethod::ZipCrypto { self.uncompressed_size() as u64 } else { self.compressed_size() as u64 };

            let mut decompression_reader: Box<dyn Read> = match local_file_header.encryption_method() {
               EncryptionMethod::NoEncryption => Box::new(zip_file_reader.take(file_size)),
               EncryptionMethod::ZipCrypto => { 
                   let zip_password = match options.zip_password() {
                       Some(pass) => pass.clone(),
                       None => return Err(ExtractError::ZipCryptoError(ZipCryptoError::InvalidPassword(String::from("Unknown Password."))))
                   };
                   let content_reader = zip_file_reader.take(file_size);
                   let zip_crypto_reader = ZipCryptoReader::new(zip_password, local_file_header.crc32(), content_reader);
                   match zip_crypto_reader {
                       Ok(reader) => Box::new(reader),
                       Err(err) => return Err(ExtractError::ZipCryptoError(err))
                   }
                },
                _ => Box::new(zip_file_reader.take(file_size))
            };
            compression_decoder::CompressionDecoder::decode_to_file(&local_file_header.compression_method(), 
                    &mut decompression_reader, 
                    &mut buf_writer)
                        .map_err(|err| ExtractError::IOError(err))?;
            Ok(Box::new(item_extract_dest_path))
        }
    }

    pub fn zip(&mut self, zip_options: &ZipOptions) -> Result<(), ZipError> {

        let file_path_on_disk = Path::new(zip_options.base_path()).join(&self.item_path);
        let mut dest_path_file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(zip_options.dest_path()).map_err(|err| ZipError::FileIOError(err))?;
        let local_file_header = local_file_header::LocalFileHeader::from_zip_item(self);

        let zip_item_start_offset = dest_path_file.seek(SeekFrom::End(0))
            .map_err(|err| ZipError::FileIOError(err))?;

        self.start_offset = zip_item_start_offset as u32;

        if self.is_file {
            let file_to_zip = File::open(file_path_on_disk)
                .map_err(|err| ZipError::FileIOError(err))?;

            let mut buf_reader = BufReader::new(file_to_zip);
            let compression_method = local_file_header.compression_method();

            let local_file_header_bin = local_file_header.to_binary();
            dest_path_file.write_all(&local_file_header_bin)
                .map_err(|err| ZipError::FileIOError(err))?;
            
            let file_data_start_offset = zip_item_start_offset + local_file_header_bin.len() as u64;

            compression_encoder::CompressionEncoder::encode_to_file(
                &compression_method,
                &mut buf_reader,
         &mut dest_path_file)
                 .map_err(|err| ZipError::FileIOError(err)
                )?;
            let file_end_offset = dest_path_file.seek(SeekFrom::Current(0))
                .map_err(|err| ZipError::FileIOError(err))?;    

            self.compressed_size = (file_end_offset - file_data_start_offset) as u32;
        } 
        else {
            dest_path_file.write_all(&local_file_header.to_binary())
                .map_err(|err| ZipError::FileIOError(err))?;
        }

        //Write Central directory file header.
        let cdfh = CentralDirectoryFileHeader::from_zip_item(self);
        let mut cdfh_bin = cdfh.to_binary();

        zip_options.update_central_directory_start_offset(dest_path_file
            .seek(SeekFrom::Current(0))
            .map_err(|err| ZipError::FileIOError(err))? as u32);

        dest_path_file.seek(SeekFrom::End(0)).map_err(|err| ZipError::FileIOError(err))?;
        dest_path_file.write_all(&mut cdfh_bin).map_err(|err| ZipError::FileIOError(err))?;

        let current_cdfh_size = zip_options.central_directory_size();
        zip_options.update_central_directory_size(current_cdfh_size + cdfh_bin.len() as u32);

        Ok(())
    }

    pub fn compressed_size(&self) -> u32 {
        self.compressed_size
    }
    pub fn start_offset(&self) -> u32 {
        self.start_offset
    }
}