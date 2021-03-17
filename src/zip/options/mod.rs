use std::{cell::Cell, ffi::OsString, path::Path};

pub struct ExtractOptions {
    zip_file_path: OsString,
    verbose_mode: bool,
    destination_path: OsString,
    zip_password: Option<String>
}

pub struct ZipOptions<'a> {
    base_path: &'a Path,
    encrypt_file: bool,
    dest_path: &'a Path,
    current_offset: u32,
    central_directory_start_offset: Cell<u32>,
    central_directory_size: Cell<u32>,

}

impl<'a> ZipOptions<'a> {

    pub fn new(base_path: &'a Path, dest_path: &'a Path, encrypt_file: bool) -> Self {
        Self {
            base_path,
            dest_path,
            encrypt_file,
            current_offset: 0,
            central_directory_start_offset: Cell::new(0),
            central_directory_size: Cell::new(0)
        }
    }

    pub fn base_path(&self) -> &Path {
        self.base_path
    }
    
    pub fn dest_path(&self) -> &Path {
        self.dest_path
    }

    pub fn encrypt_file(&self) -> bool {
        self.encrypt_file
    }

    pub fn current_offset(&self) -> u32 {
        self.current_offset
    }

    pub fn central_directory_start_offset(&self) -> u32 {
        self.central_directory_start_offset.get()
    }

    pub fn update_central_directory_start_offset(&self, new_offset: u32) {
        self.central_directory_start_offset.replace(new_offset);
    }

    pub fn central_directory_size(&self) -> u32 {
        self.central_directory_size.get()
    }

    pub fn update_central_directory_size(&self, new_offset: u32) {
        self.central_directory_size.replace(new_offset);
    }
    

}

impl ExtractOptions {
    pub fn new(verbose_mode: bool, destination_path: OsString, zip_password: Option<String>, zip_file_path: OsString) -> Self {
        Self {
            verbose_mode,
            destination_path,
            zip_password,
            zip_file_path
        }
    }

    pub fn verbose_mode(&self) -> bool {
        self.verbose_mode
    }

    pub fn destination_path(&self) -> &OsString {
        &self.destination_path
    }

    pub fn zip_password(&self) -> &Option<String> {
        &self.zip_password
    }

    pub fn zip_file_path(&self) -> &OsString {
        &self.zip_file_path
    }
}