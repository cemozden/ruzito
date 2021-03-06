use std::{path::{Path, PathBuf}, process::exit};

use clap::ArgMatches;

use crate::{cli::CommandProcessor, util, zip::{ZipFile, mem_map::EncryptionMethod, options::ExtractOptions}};


pub struct ExtractCommand;

impl CommandProcessor for ExtractCommand {
    fn command_name(&self) -> &str {
        "extract"
    }

    fn process_command(&self, matches: &ArgMatches) {

        let given_file_path = Path::new(matches.value_of(self.command_name()).unwrap());

        let file_path = if given_file_path.is_absolute() {
            let relative_path = match given_file_path.canonicalize() {
                Ok(path_buf) => path_buf,
                Err(err) => {
                    eprintln!("An error occured while canonicalizing the given zip path. Error: {}", err);
                    return;
                }
            };

            if !relative_path.exists() {
                eprintln!("Given file path does not exist!");
                return;
            }

            relative_path
        }
        else {
            PathBuf::new().join(given_file_path)
        };

        let zip_file = ZipFile::new(file_path.clone());
                
        let mut zip_file = match zip_file {
            Ok(zip_file) => zip_file,
            Err(err) => {
                eprintln!("An error occured while extracting the ZIP file! Error: {:?}", err);
                exit(-1);
            }
        };
        let destination_path = matches.value_of("dest_path")
            .map(|path| PathBuf::new().join(path));
        let destination_path = match destination_path {
            Some(dest_path) => dest_path,
            None => PathBuf::new().join(match file_path.file_name() {
                Some(file_name) => {
                    let mut path_buf = PathBuf::new().join(file_name);
                    
                    if let Some(_) = path_buf.extension() {
                        path_buf.set_extension("");
                    }

                    if !path_buf.exists() {
                        if let Err(err) = std::fs::create_dir_all(path_buf.as_path()) {
                            eprintln!("An error occured while creating the destination path folder. Error: {}", err);
                            return;
                        }
                    }
                    path_buf
                },
                None => {
                    eprintln!("An error occured while generating the destination path for extraction.");
                    return;
                }
            })
        };

        if destination_path.is_file() {
            eprintln!("Destination path of the extracted files cannot be a file path!");
            return;
        }

        let zip_password = matches.value_of("password")
            .map(|pass_str| String::from(pass_str));
        let zip_password = match zip_password {
            Some(pass) => Some(pass),
            None => if zip_file.file_encryption_method() != &EncryptionMethod::NoEncryption {
                match util::read_pass() {
                    Ok(pass) => Some(pass),
                    Err(_) => None
                }
            } else {
                None
            }
        };
        zip_file.extract_all(ExtractOptions::new(matches.is_present("verbose"),
             destination_path.as_path(),
             zip_password,
             zip_file.zip_file_path().clone()
            ));

    }

}