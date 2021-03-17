use crate::cli::CommandProcessor;

pub struct ZipCommand;

impl CommandProcessor for ZipCommand {

    fn command_name(&self) -> &str {
        "zip"
    }

    fn process_command(&self, matches: &clap::ArgMatches) {
        todo!()
    }
}