use clap::{App, Arg, ArgMatches, SubCommand};
use failure::Fallible;

use crate::imp::init as imp;

pub const COMMAND_NAME: &str = "atcoder";

pub fn clap_subcommand<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name(COMMAND_NAME).arg(Arg::with_name("NAME").required(true))
}

pub fn run(atcoder: &ArgMatches) -> Fallible<()> {
    let name = atcoder
        .value_of("NAME")
        .expect("Argument marked as required was not found.  This is a bug.");

    imp::atcoder::run(name)
}
