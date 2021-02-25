use std::{ffi::OsString, io::{Error, Write}, process::exit};

use clap::ArgMatches;

use crate::{cli::CommandProcessor, util::get_path, zip::{ZipFile, mem_map::EncryptionMethod, options::ExtractOptions}};


pub struct ExtractCommand;

impl ExtractCommand {
    #[inline]
    fn read_pass(&self) -> Result<String, Error> {
        print!("Enter password: ");
        if let Err(err) = std::io::stdout().flush() {
            return Err(err)
        }
        let pass = match rpassword::read_password() {
            Ok(pass) => pass,
            Err(err) => return Err(err)
        };

        Ok(pass)
    }

}

impl CommandProcessor for ExtractCommand {
    fn command_name(&self) -> &str {
        "extract"
    }

    fn process_command(&self, matches: &ArgMatches) {

        let file_path = matches.value_of(self.command_name()).unwrap();
        let zip_file_path = get_path(file_path);
        match zip_file_path {
            Some(path) => {
                let zip_file = ZipFile::new(path);
                
                let mut zip_file = match zip_file {
                    Ok(zip_file) => zip_file,
                    Err(err) => {
                        eprintln!("An error occured while extracting the ZIP file! Error: {:?}", err);
                        exit(-1);
                    }
                };
                let destination_path = matches.value_of("dest_path")
                    .map(|path| OsString::from(path));
                let destination_path = match destination_path {
                    Some(dest_path) => dest_path,
                    None => zip_file.zip_file_path().clone()
                };
                let zip_password = matches.value_of("password")
                    .map(|pass_str| String::from(pass_str));
                let zip_password = match zip_password {
                    Some(pass) => Some(pass),
                    None => if zip_file.file_encryption_method() != &EncryptionMethod::NoEncryption {
                        match self.read_pass() {
                            Ok(pass) => Some(pass),
                            Err(_) => None
                        }
                    } else {
                        None
                    }
                };
                zip_file.extract_all(ExtractOptions::new(matches.is_present("verbose"),
                     destination_path,
                     zip_password,
                     zip_file.zip_file_path().clone()
                    ));
            },
            None => {
                eprintln!("Given path {} is not a valid path! Exiting...", file_path);
                exit(-1);
            }
        }

    }



}