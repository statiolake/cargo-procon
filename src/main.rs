use clap::{App, AppSettings, SubCommand};
use failure::Fallible;

mod init;

fn main() -> Fallible<()> {
    let app = App::new("Assistant for participating programming contest in Rust.")
        .version("0.1.0")
        .author("statiolake <statiolake@gmail.com>")
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .subcommand(init::clap_subcommand())
        .subcommand(SubCommand::with_name("addcase"));
    let matches = app.get_matches();

    match matches.subcommand() {
        (init::INIT_COMMAND, Some(sub)) => init::run(sub),
        _ => unreachable!("Unreachable.  This is a bug."),
    }

    Ok(())
}
