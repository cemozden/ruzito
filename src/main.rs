extern crate clap;
extern crate byteorder;
extern crate inflate;
extern crate crc;

mod zip;

use std::{path::Path, process::exit};

use clap::{App, AppSettings, Arg, SubCommand};
use zip::ZipFile;

fn main() {
    let matches = App::new("ruzito")
        .version("1.0.0")
        .author("Cem Ozden <cemozden93@outlook.com>")
        .about("Simple Archive Extraction Tool")
        .settings(&[AppSettings::ArgRequiredElseHelp, AppSettings::ArgsNegateSubcommands, AppSettings::SubcommandRequiredElseHelp])
        .subcommand(SubCommand::with_name("zip")
            .arg(Arg::with_name("extract")
                    .short("x")
                    .long("extract") 
                    .value_name("ZIP_FILE")
                    .case_insensitive(true)
                    .help("Extracts the given zip file")
                    .takes_value(true) 
                )
            .arg(Arg::with_name("verbose")
                .short("v")
                .long("verbose") 
                .help("Extracts the given zip file")
                .case_insensitive(true)
                .multiple(true))
    ).get_matches();

    if let Some(matches) = matches.subcommand_matches("zip") {
        if matches.is_present("extract") {
            let zip_file_path = Path::new(matches.value_of("extract").unwrap());

            if !zip_file_path.exists() {
                let current_dir = match std::env::current_dir() {
                    Ok(path) => path,
                    Err(err) => {
                        eprintln!("An error occured! Error: {}", err);
                        exit(-1);
                    }
                };
                let zip_file_path = Path::new(current_dir.as_path()).with_file_name(zip_file_path);

                if zip_file_path.exists() {
                    let zip_file = ZipFile::new(zip_file_path);

                    let mut zip_file = match zip_file {
                        Ok(zip_file) => zip_file,
                        Err(err) => {
                            eprintln!("An error occured while extracting the ZIP file! Error: {:?}", err);
                            exit(-1);
                        }
                    };

                    zip_file.extract_all();
                }
                else {
                    eprintln!(r"Unable to find the given path {:?}", zip_file_path);
                }
            }
            else {
                let zip_file = ZipFile::new(zip_file_path);

                let mut zip_file = match zip_file {
                        Ok(zip_file) => zip_file,
                        Err(err) => {
                            eprintln!("An error occured while extracting the ZIP file! Error: {:?}", err);
                            exit(-1);
                        }
                    };

                    zip_file.extract_all();
            }
        }
    }

}