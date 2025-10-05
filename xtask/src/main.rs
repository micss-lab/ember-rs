use anyhow::Result;
use clap::Parser;

use self::cargo::Cargo;

const LOGGER_ENV: &str = "EMBER_XTASK_LOG";

mod cargo;

#[derive(Parser)]
enum Cli {
    Build,
    Check,
}

fn main() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().filter_or(LOGGER_ENV, "info"))
        .target(env_logger::Target::Stdout)
        .init();

    match Cli::parse() {
        Cli::Build => Cargo::default().build(),
        Cli::Check => Cargo::default().check(),
    }
}
