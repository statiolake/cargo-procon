use clap::{App, Arg, ArgMatches, SubCommand};
use failure::Fallible;

use crate::imp::testcase as imp;

pub const COMMAND_NAME: &str = "delcase";

pub fn clap_subcommand<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name(COMMAND_NAME).arg(Arg::with_name("NAME").required(true))
}

pub fn run(matches: &ArgMatches) -> Fallible<()> {
    let name = matches
        .value_of("NAME")
        .expect("Required argument does not found.  This is a bug.");

    imp::delcase(&name)
}
