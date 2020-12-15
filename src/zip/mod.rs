use std::path::Path;

mod mem_map;
mod local_file_header;
mod eof_central_dir;
mod central_dir_file_header;
mod archive_extra_data_record;
mod data_descriptor;
mod zip_metadata;
mod date_time;
pub mod zip_item;

pub enum ZipError {
    FileIOError(std::io::Error)
}
pub struct ZipFile {
    file_count: u16,
    zip_items: Vec<zip_item::ZipItem>
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
        let (eof_central_dir, file_headers) = zip_metadata::ZipMetadata::parse(zip_file_path)
            .map_err(|io_error| ZipError::FileIOError(io_error))?;
        
        let zip_items = file_headers.into_iter()
            .map(|item| item.into())
            .collect();
            
        Ok(ZipFile {
            zip_items, 
            file_count: eof_central_dir.total_num_of_central_dir()
        })
    }

    pub fn file_count(&self) -> u16 {
        self.file_count
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