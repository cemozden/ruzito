use std::{ffi::OsString, path::Path};
use crate::{util, cli::CommandProcessor, zip::{ZipFile, mem_map::EncryptionMethod, options::ZipOptions, zip_item_creator::ZipItemCreator}};

const MIN_ZIP_ITEM_CAPACITY: usize = 50;

pub struct ZipCommand;

impl CommandProcessor for ZipCommand {

    fn command_name(&self) -> &str {
        "zip"
    }

    fn process_command(&self, matches: &clap::ArgMatches) {
        let zip_path = Path::new(matches.value_of(self.command_name()).unwrap());
        let dest_path = Path::new(match matches.value_of("dest_path") {
            Some(p) => p,
            None => return
        });

        let encrypt_file = matches.is_present("encrypt");
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
        let zip_options = ZipOptions::new(zip_path, dest_path, encrypt_file, zip_password, verbose_mode);
        
        let zip_item_creator = ZipItemCreator::new(zip_path);

        if verbose_mode {
            println!("Finding items to be zipped.");
        }
        if let Err(err) = zip_item_creator.create_zip_items(zip_path, None, &mut zip_items, encryption_method) {
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