use std::{ffi::{OsStr, OsString}, fs::{Metadata, OpenOptions, read_dir}, io::{Error, ErrorKind, Seek, SeekFrom, Write}, os::windows::prelude::MetadataExt, path::Path, time::SystemTime};

use chrono::{DateTime, Datelike, Local, Timelike};

use super::{ZipCreatorError, ZipFile, crc32::calculate_checksum, date_time::ZipDateTime, eof_central_dir::EndOfCentralDirectory, mem_map::CompressionMethod, options::ZipOptions, zip_item::ZipItem};
use super::mem_map::EncryptionMethod;

const MIN_ZIP_ITEM_CAPACITY: usize = 10;
const MIN_SIZE_TO_COMPRESS: u64 = 10000;

pub struct ZipCreator<'a>{
    base_path: &'a Path
}

impl<'a> ZipCreator<'a> {

    pub fn new(base_path: &'a Path) -> Self {
        Self {
            base_path
        }
    }

    pub fn create(&self, zip_options: &ZipOptions) -> Result<ZipFile, ZipCreatorError> {
        let path = self.base_path;

        if !path.exists() {
            return Err(ZipCreatorError::InvalidInPath(path.as_os_str().to_owned()))
        }

        let mut zip_items = Vec::with_capacity(MIN_ZIP_ITEM_CAPACITY);
        self.create_zip_items(path, None, &mut zip_items)?;
        let item_count = zip_items.len() as u16;

        for zip_item in &mut zip_items {
            println!("{}", zip_item.item_path());

            if let Err(err) =  zip_item.zip(zip_options) {
                return Err(ZipCreatorError::ZipError(err))
            }
        }

        let eocd = EndOfCentralDirectory::from_zip_creator(zip_items.len() as u16, zip_options.central_directory_size(), zip_options.central_directory_start_offset());
        let mut  eocd_bytes = eocd.to_binary(); 
        let mut dest_file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(zip_options.dest_path()).map_err(|err| ZipCreatorError::IOError(err))?;

        dest_file.seek(SeekFrom::End(0)).map_err(|err| ZipCreatorError::IOError(err))?;
        dest_file.write_all(&mut eocd_bytes).map_err(|err| ZipCreatorError::IOError(err))?;

        Ok(
            ZipFile::create(
                item_count,
                 zip_items.into_iter().collect(),
             OsString::from(zip_options.dest_path().as_os_str()),
            if zip_options.encrypt_file() { EncryptionMethod::ZipCrypto } else { EncryptionMethod::NoEncryption }
            )
        )
        
    }

    fn create_zip_items(&self, path: &Path, item_path: Option<&OsStr>, zip_items: &mut Vec<ZipItem>) -> Result<(), ZipCreatorError> {

        if path.is_dir() {
           if let Some(it_path) = item_path {

               let mut zip_item_path = OsString::from(it_path).into_string().map_err(|os_string| ZipCreatorError::InvalidPath(os_string))?.replace(r"\", "/");                 
               zip_item_path.push('/');

               let directory = std::fs::metadata(path)
                    .map_err(|err| ZipCreatorError::IOError(err))?;
            
               zip_items.push(ZipItem::new(
                   CompressionMethod::NoCompression, 
                   zip_item_path,
                   0, 
                   0, 
                   self.get_file_modified_date_time(&directory)
                    .map_err(|err| ZipCreatorError::IOError(err))?, 
                   0, 
                   EncryptionMethod::NoEncryption,
                   0)
               )
           }

           let dir_content = read_dir(path).map_err(|err| ZipCreatorError::IOError(err))?;

           for entry in dir_content {
                let entry = match entry {
                    Ok(entry) => entry,
                    Err(err) => return Err(ZipCreatorError::IOError(err))
                };

                let entry_path = entry.path();
                let item_path = entry_path.strip_prefix(self.base_path)
                    .map_err(|_| ZipCreatorError::InvalidPath(OsString::from("Unable to apply strip prefix!")))?;
                
                self.create_zip_items(entry_path.as_path(), Some(item_path.as_os_str()), zip_items)?;
           }

        }
        else {

            if !path.is_file() {
                return Err(ZipCreatorError::InvalidPath(OsString::from(format!("The path {} does not exist!", path.display()))));
            }

            let item_path = match item_path {
                Some(path) => path,
                None => path.file_name().unwrap()
            };
            let file_metadata = std::fs::metadata(path).map_err(|err| ZipCreatorError::IOError(err))?;

            let file_size = file_metadata.file_size();
            let compression_method = if file_size > MIN_SIZE_TO_COMPRESS {
                CompressionMethod::Deflate
            }
            else {
                CompressionMethod::NoCompression
            };

            let zip_item_path = OsString::from(item_path).into_string().map_err(|os_string| ZipCreatorError::InvalidPath(os_string))?.replace(r"\", "/");

            zip_items.push(ZipItem::new(
                compression_method,
                zip_item_path,
                file_size as u32,
                0,
                self.get_file_modified_date_time(&file_metadata).map_err(|err| ZipCreatorError::IOError(err))?,
                0,
                EncryptionMethod::NoEncryption,
                calculate_checksum(path).map_err(|err| ZipCreatorError::IOError(err))?
            ));
            
        }

        Ok(())
    }

    fn get_file_modified_date_time(&self, metadata: &Metadata) -> Result<ZipDateTime, Error> {

        let modified = metadata.modified()?;
        let modified_date_time = SystemTime::UNIX_EPOCH + modified.duration_since(SystemTime::UNIX_EPOCH)
            .map_err(|err| Error::new(ErrorKind::Interrupted, err))?;

        let datetime = DateTime::<Local>::from(modified_date_time);

        let day = datetime.date().day();
        let month = datetime.date().month();
        let year = datetime.date().year();

        let hour = datetime.time().hour();
        let minutes = datetime.time().minute();
        let seconds = datetime.time().second();

        Ok(ZipDateTime::new(day as u8, month as u8, year as u16, hour as u8, minutes as u8, seconds as u8))
    }


}