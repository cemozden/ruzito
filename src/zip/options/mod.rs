use std::{ffi::OsString};

pub struct ExtractOptions {
    verbose_mode: bool,
    destination_path: OsString,
    zip_password: Option<String>
}

impl ExtractOptions {
    pub fn new(verbose_mode: bool, destination_path: OsString, zip_password: Option<String>) -> Self {
        Self {
            verbose_mode,
            destination_path,
            zip_password
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
}