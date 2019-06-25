use clap::{App, AppSettings, ArgMatches, SubCommand};
use failure::Fallible;

pub const COMMAND_NAME: &str = "create";

mod atcoder;

pub fn clap_subcommand<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name(COMMAND_NAME)
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .subcommand(atcoder::clap_subcommand())
}

pub fn run(matches: &ArgMatches) -> Fallible<()> {
    match matches.subcommand() {
        (atcoder::COMMAND_NAME, Some(atcoder)) => atcoder::run(atcoder),
        _ => unreachable!("This is a bug."),
    }
}
