use clap::{App, AppSettings, ArgMatches, SubCommand};

pub const INIT_COMMAND: &str = "init";

const SC_ATCODER: &str = "atcoder";
const SC_GENERAL: &str = "general";

pub fn clap_subcommand<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name(INIT_COMMAND)
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .subcommand(SubCommand::with_name(SC_GENERAL))
        .subcommand(SubCommand::with_name(SC_ATCODER))
}

pub fn run(matches: &ArgMatches) {
    match matches.subcommand() {
        (SC_GENERAL, Some(general)) => run_general(general),
        (SC_ATCODER, Some(atcoder)) => run_atcoder(atcoder),
        _ => unreachable!("This is a bug."),
    }
}

fn run_general(_general: &ArgMatches) {}

fn run_atcoder(_atcoder: &ArgMatches) {}
