use clap::{App, AppSettings};
use failure::Fallible;

pub mod addcase;
pub mod create;

pub fn main() -> Fallible<()> {
    let app = App::new("Assistant for participating programming contest in Rust.")
        .version("0.1.0")
        .author("statiolake <statiolake@gmail.com>")
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .subcommand(create::clap_subcommand())
        .subcommand(addcase::clap_subcommand());
    let matches = app.get_matches();

    match matches.subcommand() {
        (create::COMMAND_NAME, Some(sub)) => create::run(sub),
        (addcase::COMMAND_NAME, Some(sub)) => addcase::run(sub),
        _ => unreachable!("Unreachable.  This is a bug."),
    }
}
