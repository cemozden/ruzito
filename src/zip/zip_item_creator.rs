use std::{ffi::{OsStr, OsString}, fs::{Metadata, read_dir}, io::{Error, ErrorKind}, path::PathBuf, time::SystemTime};

use chrono::{DateTime, Datelike, Local, Timelike};

use super::{ZipCreatorError, crc32::calculate_checksum, date_time::ZipDateTime, mem_map::CompressionMethod, zip_item::ZipItem};
use super::mem_map::EncryptionMethod;

const MIN_SIZE_TO_COMPRESS: u64 = 10000;

pub struct ZipItemCreator<'a>{
    base_path: &'a PathBuf
}

impl<'a> ZipItemCreator<'a> {

    pub fn new(base_path: &'a PathBuf) -> Self {
        Self {
            base_path
        }
    }

    pub fn create_zip_items(&self, path: &PathBuf, item_path: Option<&OsStr>, zip_items: &mut Vec<ZipItem>, encryption_method: EncryptionMethod) -> Result<(), ZipCreatorError> {

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
                
                self.create_zip_items(&entry_path, Some(item_path.as_os_str()), zip_items, encryption_method)?;
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

            let file_size = file_metadata.len();
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
                encryption_method,
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