use std::env;
use std::process::Command;

use anyhow::{Context, Result, bail};

/// Runs `cargo <args>`, inheriting stdio. Resolves the cargo binary via the
/// `CARGO` env var (set by cargo itself when invoking `cargo run`), falling
/// back to plain `"cargo"` when unset.
pub fn run_cargo(args: &[&str]) -> Result<()> {
    let cargo = env::var("CARGO").unwrap_or_else(|_| "cargo".into());
    let joined = args.join(" ");

    // Unconditional (not gated by XTASK_LOG) and on stderr, so the exact
    // command is always visible — for both copy-paste retries outside xtask,
    // and to keep stdout clean for commands whose output feeds a parser
    // (`lsp`'s concatenated cargo JSON).
    eprintln!("$ {cargo} {joined}");

    let status = Command::new(&cargo)
        .args(args)
        .status()
        .with_context(|| format!("failed to execute `{cargo} {joined}`"))?;

    if !status.success() {
        bail!("`{cargo} {joined}` exited with {status}");
    }

    Ok(())
}
