extern crate clap;
extern crate byteorder;
extern crate inflate;
extern crate flate2;
extern crate crc;
extern crate cli_table;
extern crate rpassword;
extern crate chrono;
extern crate rand;

mod zip;
mod cli;
mod util;

use clap::{App, AppSettings};
use cli::SubCommandModule;

fn main() {

    let commands = SubCommandModule::new();

    let matches = App::new("ruzito")
        .version("1.0.0")
        .author("Cem Ozden <cemozden93@outlook.com>")
        .about("Simple Archive Extraction Tool")
        .settings(&[AppSettings::ArgRequiredElseHelp, AppSettings::ArgsNegateSubcommands, AppSettings::SubcommandRequiredElseHelp])
        .subcommands(commands.sub_commands())
        .get_matches();

    commands.run(matches);
}