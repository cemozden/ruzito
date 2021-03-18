use std::path::Path;

use crate::{cli::CommandProcessor, zip::{options::ZipOptions, zip_creator::ZipCreator, ZipCreatorError}};

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
        let zip_options = ZipOptions::new(zip_path, dest_path, false);
        let zip_creator = ZipCreator::new(zip_path);

        match zip_creator.create(&zip_options) {
            Ok(_) => {
                //TODO: Implement successful message
            },
            Err(err) => {
                match err {
                    ZipCreatorError::InvalidInPath(path) => eprintln!("Invalid path to zip! Path: {:?}", path),
                    ZipCreatorError::InvalidPath(path) => eprintln!("Invalid path provided! Given Path: {:?}", path),
                    ZipCreatorError::IOError(err) => eprintln!("An I/O error occured! Error: {:?}", err),
                    ZipCreatorError::ZipError(err) => eprintln!("A Zip error occurd! Error: {:?}", err)                }
            }
        }

    }
}