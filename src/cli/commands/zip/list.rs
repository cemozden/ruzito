use std::process::exit;

use clap::ArgMatches;
use cli_table::{Cell, CellStruct, Table, format::Justify, print_stdout};

use crate::{cli::CommandProcessor, util::get_path, zip::{ZipFile, mem_map::EncryptionMethod}};

pub struct ListCommand;

type TableRow = Vec<CellStruct>;

impl CommandProcessor for ListCommand {
    fn command_name(&self) -> &str {
        "list"
    }

    fn process_command(&self, matches: &ArgMatches) {
        if !matches.is_present(self.command_name()) { return; }
        
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
