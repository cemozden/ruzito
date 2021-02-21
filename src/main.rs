extern crate clap;
extern crate byteorder;
extern crate inflate;
extern crate crc;
extern crate cli_table;
extern crate rpassword;

mod zip;

use std::{ffi::{OsString}, io::{Error, Write}, path::Path, process::exit};
use clap::{App, AppSettings, Arg, SubCommand};
use zip::{ZipFile, mem_map::EncryptionMethod, options::ExtractOptions};
use cli_table::{Cell, CellStruct, Table, format::Justify, print_stdout};

type TableRow = Vec<CellStruct>;

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
                .help("Print the extracted files during extracting stage")
                .case_insensitive(true))
            .arg(Arg::with_name("dest_path")
                .short("d")
                .long("destination-path") 
                .help("The path where ZIP files will be extracted")
                .takes_value(true)
                .value_name("PATH")
                .case_insensitive(true))
            .arg(Arg::with_name("password")
                .short("p")
                .long("password") 
                .help("The password of the ZIP file. If it's not provided, then ruzito will ask for it if required.")
                .takes_value(true)
                .value_name("PASSWORD")
                .case_insensitive(true))
            .arg(Arg::with_name("list")
                .short("l")
                .long("list")
                .help("Lists the files/directories inside of the ZIP file")
                .case_insensitive(true)
                .takes_value(true)
                .value_name("ZIP_FILE")

        )
    ).get_matches();

    if let Some(matches) = matches.subcommand_matches("zip") {
        if matches.is_present("extract") {
            let file_path = matches.value_of("extract").unwrap();
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
                            match read_pass() {
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

        if matches.is_present("list") {
            let file_path = matches.value_of("list").unwrap();
            let zip_file_path = get_path(file_path);

            match zip_file_path {
                Some(path) => {
                    let zip_file = match ZipFile::new(path) {
                        Ok(zip_file) => zip_file,
                        Err(err) => {
                            eprintln!("An error occured while extracting the ZIP file! Error: {:?}", err);
                            exit(-1)
                        }
                    };

                    let list_table = zip_file.iter()
                        .map(|item| {

                            let compression_perc = if item.uncompressed_size() > 0 {
                                let compressed_size = item.compressed_size() as f32;
                                let uncompressed_size = item.uncompressed_size() as f32;

                                let perc = ((compressed_size / uncompressed_size) * 100.0) as f32;
                                format!("({:.1}%)", perc)
                            }
                            else { String::from("") };

                            let file_protected = if item.encryption_method() == EncryptionMethod::NoEncryption {
                                "No"
                            } else {
                                "Yes"
                            };


                            vec![
                                item.item_path().cell(),
                                format!("{:?} {}", item.compression_method(), compression_perc).cell(),
                                item.compressed_size().cell().justify(Justify::Right),
                                file_protected.cell(),
                                item.uncompressed_size().cell().justify(Justify::Right),
                                format!("{}", item.modified_date_time()).cell()
                            ]}
                        )
                        .collect::<Vec<TableRow>>()
                        .table()
                        .title(vec![
                          "Item".cell(),
                          "Compression".cell(),
                          "Compressed Size".cell(),
                          "Password Protected".cell(),
                          "File Size".cell(),
                          "Modified Date".cell()
                        ]);

                      if let Err(err) = print_stdout(list_table) {                            
                          eprintln!("An error occured while creating the table. {}", err);
                          exit(-1);
                      }

                      println!("\n{} files/directories listed.\n", zip_file.file_count());

                },
                None => {
                    eprintln!("Given path {} is not a valid path! Exiting...", file_path);
                    exit(-1);
                }
            }



        }
    }

}
#[inline]
pub fn read_pass() -> Result<String, Error> {
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

fn get_path<P>(path: P) -> Option<OsString> where P: AsRef<Path> {
    let file_path = path.as_ref();

     if !file_path.exists() {
           let current_dir = match std::env::current_dir() {
               Ok(path) => path,
               Err(err) => {
                   eprintln!("An error occured! Error: {}", err);
                   return None;
               }
           };
           let file_path = Path::new(current_dir.as_path()).with_file_name(file_path);

           if file_path.exists() {
               return Some(OsString::from(file_path.as_os_str()));
           }
           else {
               eprintln!(r"Unable to find the given path {:?}", file_path);
               return None;
           }
       }
       else {
           return Some(OsString::from(file_path.as_os_str()));
       }

}