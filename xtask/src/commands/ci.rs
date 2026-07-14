use anyhow::{Result, bail};

use crate::commands::{check, clippy, hack, test};

type Step = fn() -> Result<()>;

/// Runs every check without bailing on the first failure, so a single `ci`
/// run gives a full picture instead of stopping at the first red step.
pub fn run() -> Result<()> {
    let steps: [(&str, Step); 4] = [
        ("check", || check::run(None)),
        ("clippy", || clippy::run(None, false)),
        ("hack", || hack::run(None)),
        ("test", test::run),
    ];

    let mut failed = Vec::new();

    for (name, step) in steps {
        log::info!("=== xtask ci: {name} ===");
        if let Err(err) = step() {
            log::error!("{name} failed: {err:#}");
            failed.push(name);
        }
    }

    println!("\nxtask ci summary:");
    for (name, _) in steps {
        let status = if failed.contains(&name) { "FAIL" } else { "ok" };
        println!("  {status:<4} {name}");
    }

    if !failed.is_empty() {
        bail!("xtask ci: {} step(s) failed: {}", failed.len(), failed.join(", "));
    }

    Ok(())
}
