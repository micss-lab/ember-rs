use anyhow::Result;
use clap::{Parser, Subcommand};

use crate::commands;
use crate::target::Target;

#[derive(Debug, Parser)]
#[command(name = "xtask", about = "Developer tooling for the ember-rs workspace")]
pub struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Build the workspace, or a single binary, for one or both targets.
    Build {
        /// Which target to build for. Defaults to both.
        #[arg(long)]
        target: Option<Target>,
        /// Build only this binary (e.g. an example under examples/src/bin).
        #[arg(long)]
        bin: Option<String>,
        /// Extra arguments forwarded verbatim to the underlying cargo invocation.
        #[arg(last = true)]
        args: Vec<String>,
    },
    /// Build, then flash (ESP32) or execute (local), a single binary.
    Run {
        /// Which target to run on.
        #[arg(long)]
        target: Target,
        /// The binary to run (e.g. an example under examples/src/bin).
        #[arg(long)]
        bin: String,
        /// Extra arguments forwarded verbatim to the underlying cargo invocation.
        #[arg(last = true)]
        args: Vec<String>,
    },
    /// `cargo check`. Defaults to both targets.
    Check {
        /// Restrict to a single target instead of checking both.
        #[arg(long)]
        target: Option<Target>,
    },
    /// `cargo clippy`. Defaults to both targets.
    Clippy {
        /// Restrict to a single target instead of linting both.
        #[arg(long)]
        target: Option<Target>,
        /// Apply clippy's suggested fixes (implies --allow-dirty, since
        /// requiring a clean git tree first is rarely what you want here).
        #[arg(long)]
        fix: bool,
    },
    /// `cargo test`, local target only.
    Test,
    /// `cargo hack check --feature-powerset`. Defaults to both targets.
    Hack {
        /// Restrict to a single target instead of hacking both.
        #[arg(long)]
        target: Option<Target>,
    },
    /// Emit combined clippy JSON diagnostics for both targets (for rust-analyzer's overrideCommand).
    Lsp,
    /// Run check, clippy, hack, and test; report a summary.
    Ci,
}

impl Cli {
    pub fn run(self) -> Result<()> {
        match self.command {
            Command::Build { target, bin, args } => commands::build::run(target, bin, &args),
            Command::Run { target, bin, args } => commands::run::run(target, &bin, &args),
            Command::Check { target } => commands::check::run(target),
            Command::Clippy { target, fix } => commands::clippy::run(target, fix),
            Command::Test => commands::test::run(),
            Command::Hack { target } => commands::hack::run(target),
            Command::Lsp => commands::lsp::run(),
            Command::Ci => commands::ci::run(),
        }
    }
}
