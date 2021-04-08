use std::{ffi::OsString, path::Path};
use std::path::PathBuf;
use std::fs::File;
use crate::{util, cli::CommandProcessor, zip::{ZipFile, mem_map::EncryptionMethod, options::ZipOptions, zip_item_creator::ZipItemCreator}};

const MIN_ZIP_ITEM_CAPACITY: usize = 50;

pub struct ZipCommand;

impl CommandProcessor for ZipCommand {

    fn command_name(&self) -> &str {
        "zip"
    }

    fn process_command(&self, matches: &clap::ArgMatches) {
        let mut zip_file_name;
        let given_zip_path = Path::new(matches.value_of(self.command_name()).unwrap());
        let given_dest_path = Path::new(match matches.value_of("dest_path") {
            Some(p) => p,
            None => {
                match given_zip_path.file_name() {
                     Some(file_name) => {
                         zip_file_name = OsString::from(file_name);
                         zip_file_name.push(".zip");

                         match zip_file_name.to_str() {
                             Some(path) => path,
                             None => {
                                 eprintln!("An error occured while generating destination path. Exiting..");
                                 return;
                             }
                         }
                     },
                     None => {
                         eprintln!("Invalid zip path. Exiting...");
                         return;
                     }
            }
        }
        });

        let zip_path = if given_zip_path.is_absolute() {
            let relative_path = match given_zip_path.canonicalize() {
                Ok(path_buf) => path_buf,
                Err(err) => {
                    eprintln!("An error occured while canonicalizing the given zip path. Error: {}", err);
                    return;
                }
            };

            if !relative_path.exists() {
                eprintln!("Given zip path does not exist!");
                return;
            }
            
            relative_path
        }
        else { PathBuf::new().join(given_zip_path) };

        let dest_path = if given_dest_path.is_absolute() {
            if !given_dest_path.exists() {
                if let Err(err) = File::create(given_dest_path) {
                    eprintln!("An error occured while creating destination zip file. Error: {}", err);
                    return;
                }
            }

            let relative_path = match given_dest_path.canonicalize() {
                Ok(path_buf) => path_buf,
                Err(err) => {
                    eprintln!("An error occured while canonicalizing the given destination path. Error: {}", err);
                    return;
                }
            };

            relative_path
        } else { PathBuf::new().join(given_dest_path) };

        let encrypt_file = matches.is_present("encrypt") || matches.is_present("password");
        let verbose_mode = matches.is_present("verbose");

        let encryption_method = if encrypt_file { EncryptionMethod::ZipCrypto } else { EncryptionMethod::NoEncryption };

        let zip_password = matches.value_of("password")
            .map(|pass_str| String::from(pass_str));

        let zip_password = match zip_password {
            Some(pass) => Some(pass),
            None => if encrypt_file {
                match util::read_pass() {
                    Ok(pass) => Some(pass),
                    Err(_) => None
                }
            } else {
                None
            }
        };

        let mut zip_items = Vec::with_capacity(MIN_ZIP_ITEM_CAPACITY);
        let zip_options = ZipOptions::new(&zip_path, &dest_path, encrypt_file, zip_password, verbose_mode);
        
        let zip_item_creator = ZipItemCreator::new(&zip_path);

        if verbose_mode {
            println!("Finding items to be zipped.");
        }
        if let Err(err) = zip_item_creator.create_zip_items(&zip_path, None, &mut zip_items, encryption_method) {
            eprintln!("An error occured while creating zip items! Err: {:?}", err);
            return;
        }

        if verbose_mode {
            println!("Finding items completed. {} items found.", zip_items.len());
        }

        let mut zip_file = ZipFile::create(zip_items.len() as u16, zip_items, OsString::from(dest_path.as_os_str()), encryption_method);

        match zip_file.create_zip_file(&zip_options) {
            Ok(()) => {
                //TODO: Successful message.
            },
            Err(err) => eprintln!("An error occured while zipping the path Error: {:?}", err)
        };

    }
}