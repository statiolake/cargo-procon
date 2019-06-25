use failure::ensure;
use failure::Fallible;
use std::process::Command;

const TEMPLATE_GIT_URL: &str = "https://github.com/rust-lang-ja/atcoder-rust-base";
const TEMPLATE_GIT_BRANCH: Option<&str> = Some("ja");

pub fn run(name: &str) -> Fallible<()> {
    ensure!(
        which::which("cargo").is_ok(),
        "Failed to find cargo.  Please install cargo first."
    );
    ensure!(
        which::which("cargo-generate").is_ok(),
        "Failed to find cargo-generate.  \
         If you hadn't install it, run `cargo install cargo-generate` in your terminal."
    );

    let mut cmd = Command::new("cargo");
    cmd.arg("generate");

    cmd.arg("-n").arg(name);
    cmd.arg("--git").arg(TEMPLATE_GIT_URL);
    if let Some(branch) = TEMPLATE_GIT_BRANCH {
        cmd.arg("--branch").arg(branch);
    }

    let status = cmd.spawn()?.wait()?;
    ensure!(status.success(), "Running cargo-generate failed.");

    Ok(())
}
