use std::{ffi::OsString, path::Path};
use crate::{cli::CommandProcessor, zip::{ZipFile, mem_map::EncryptionMethod, options::ZipOptions, zip_item_creator::ZipItemCreator}};

const MIN_ZIP_ITEM_CAPACITY: usize = 10;

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
        let mut zip_items = Vec::with_capacity(MIN_ZIP_ITEM_CAPACITY);

        let zip_options = ZipOptions::new(zip_path, dest_path, false);
        let zip_item_creator = ZipItemCreator::new(zip_path);

        if let Err(err) = zip_item_creator.create_zip_items(zip_path, None, &mut zip_items) {
            eprintln!("An error occured while creating zip items! Err: {:?}", err);
            return;
        }

        let mut zip_file = ZipFile::create(zip_items.len() as u16, zip_items, OsString::from(dest_path.as_os_str()), EncryptionMethod::NoEncryption);

        match zip_file.create_zip_file(&zip_options) {
            Ok(()) => {
                //TODO: Successful message.
            },
            Err(err) => eprintln!("An error occured while zipping the path Error: {:?}", err)
        };

    }
}