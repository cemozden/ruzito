use clap::{Arg, ArgMatches, SubCommand};

use crate::cli::{CommandProcessor, RuzitoSubCommand};

mod extract;
mod list;

pub struct ZipSubCommand {
    commands: Vec<Box<dyn CommandProcessor>>
}

impl ZipSubCommand {

    pub fn new() -> Self {

        Self {
            commands: vec![
                Box::new(extract::ExtractCommand),
                Box::new(list::ListCommand)
            ]
        }

    }
}

impl RuzitoSubCommand for ZipSubCommand {
    fn clap_definition<'a, 'b>(&self) -> clap::App<'a, 'b> {
        SubCommand::with_name(self.name())
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
                .help("The password of the ZIP file.")
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
    }

    fn run_command_processes(&self, matches: &ArgMatches) {
        if let Some(matches) = matches.subcommand_matches(self.name()) { 
            self.commands.iter()
                .for_each(|command_processor| command_processor.process_command(matches));
         }

    }

    fn name(&self) -> &str {
        "zip"   
    }
}