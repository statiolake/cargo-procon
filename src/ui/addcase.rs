use clap::{App, Arg, ArgMatches, SubCommand};
use failure::Fallible;

use crate::imp::testcase as imp;

pub const COMMAND_NAME: &str = "addcase";

pub fn clap_subcommand<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name(COMMAND_NAME).arg(Arg::with_name("NAME"))
}

pub fn run(matches: &ArgMatches) -> Fallible<()> {
    let default_id = imp::next_id()?;
    let name = matches.value_of("NAME").unwrap_or(&default_id);

    let input = read("input")?;
    let output = read("output")?;

    imp::addcase(&name, false, &input, &output)
}

fn read(prompt: &str) -> Fallible<String> {
    use std::io::prelude::*;
    use std::io::stdin;
    println!("{}:", prompt);
    let mut input = String::new();
    stdin().read_to_string(&mut input)?;

    Ok(input)
}
