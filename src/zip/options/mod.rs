use std::{ffi::OsString, path::{Path, PathBuf}};

pub struct ExtractOptions<'a> {
    zip_file_path: OsString,
    verbose_mode: bool,
    destination_path: &'a Path,
    zip_password: Option<String>
}

pub struct ZipOptions<'a> {
    base_path: &'a PathBuf,
    encrypt_file: bool,
    dest_path: &'a PathBuf,
    password: Option<String>,
    verbose_mode: bool
}

impl<'a> ZipOptions<'a> {

    pub fn new(base_path: &'a PathBuf, dest_path: &'a PathBuf, encrypt_file: bool, password: Option<String>, verbose_mode: bool) -> Self {
        Self {
            base_path,
            dest_path,
            encrypt_file,
            password,
            verbose_mode
        }
    }

    pub fn base_path(&self) -> &PathBuf {
        self.base_path
    }
    
    pub fn dest_path(&self) -> &PathBuf {
        self.dest_path
    }

    pub fn password(&self) -> &Option<String> {
        &self.password
    }

    pub fn encrypt_file(&self) -> bool {
        self.encrypt_file
    }

    pub fn verbose_mode(&self) -> bool {
        self.verbose_mode
    }

}

impl<'a> ExtractOptions<'a> {
    pub fn new(verbose_mode: bool, destination_path: &'a Path, zip_password: Option<String>, zip_file_path: OsString) -> Self {
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

    pub fn destination_path(&self) -> &Path {
        self.destination_path
    }

    pub fn zip_password(&self) -> &Option<String> {
        &self.zip_password
    }

    pub fn zip_file_path(&self) -> &OsString {
        &self.zip_file_path
    }
}