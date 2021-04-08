use std::process::exit;
use std::path::{Path, PathBuf};
use clap::ArgMatches;
use cli_table::{Cell, CellStruct, Table, format::Justify, print_stdout};

use crate::{cli::CommandProcessor, zip::{ZipFile, mem_map::{EncryptionMethod, CompressionMethod}}};

pub struct ListCommand;

type TableRow = Vec<CellStruct>;

impl CommandProcessor for ListCommand {
    fn command_name(&self) -> &str {
        "list"
    }

    fn process_command(&self, matches: &ArgMatches) {
        
        let given_file_path = Path::new(matches.value_of("list").unwrap());

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

        let zip_file = match ZipFile::new(file_path) {
            Ok(zip_file) => zip_file,
            Err(err) => {
                eprintln!("An error occured while extracting the ZIP file! Error: {:?}", err);
                exit(-1)
            }
        };

        let list_table = zip_file.iter()
            .map(|item| {
                let compression_perc = if item.uncompressed_size() > 0 && item.compression_method() != CompressionMethod::NoCompression {
                    let compressed_size = item.compressed_size() as f32;
                    let uncompressed_size = item.uncompressed_size() as f32;
                    let perc = ((compressed_size / uncompressed_size) * 100.0) as f32;
                    format!("({:.1}%)", 100 as f32 - perc)
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

    }
}
