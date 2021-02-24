use clap::{App, ArgMatches};

mod commands;
use cli::commands::zip::ZipSubCommand;
pub struct SubCommandModule {
    sub_commands: Vec<Box<dyn RuzitoSubCommand>>
}

impl SubCommandModule {
    pub fn new() -> Self {
        Self {
            sub_commands: vec![
                Box::new(ZipSubCommand::new())
            ]
        }
    }

    pub fn sub_commands<'a, 'b>(&self) -> Vec<App<'a, 'b>> {
        self.sub_commands.iter()
            .map(|sub_command| sub_command.clap_definition())
            .collect()
    }

    pub fn run(&self, matches: ArgMatches) {
       self.sub_commands.iter()
        .for_each(|sub_command| sub_command.run_command_processes(&matches)); 
    }
}

pub trait RuzitoSubCommand {
    fn clap_definition<'a, 'b>(&self) -> App<'a, 'b>;
    fn name(&self) -> &str;
    fn run_command_processes(&self, matches: &ArgMatches); 
}

pub trait CommandProcessor {
    fn command_name(&self) -> &str;
    fn process_command(&self, matches: &ArgMatches);
}