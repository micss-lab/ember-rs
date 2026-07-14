use anyhow::Result;
use clap::Parser;

use crate::cli::Cli;

const LOGGER_ENV: &str = "XTASK_LOG";

mod cargo;
mod cli;
mod commands;
mod features;
mod metadata;
mod target;

fn main() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().filter_or(LOGGER_ENV, "info"))
        .target(env_logger::Target::Stderr)
        .init();

    Cli::parse().run()
}
