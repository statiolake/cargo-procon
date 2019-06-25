use clap::{App, ArgMatches, SubCommand};

pub const INIT_COMMAND: &str = "init";

pub fn clap_subcommand<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name(INIT_COMMAND)
}

pub fn run(_matches: &ArgMatches) {
    println!("init");
}
