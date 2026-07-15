use std::env;
use std::process::Command;

use anyhow::{Context, Result, bail};
use serde::Deserialize;

#[derive(Deserialize)]
struct WorkspaceMetadata {
    packages: Vec<Package>,
}

#[derive(Deserialize)]
struct Package {
    name: String,
    targets: Vec<BinTarget>,
}

#[derive(Deserialize)]
struct BinTarget {
    name: String,
    kind: Vec<String>,
}

/// Finds which workspace package owns the binary target named `bin`.
///
/// Resolved dynamically (rather than assuming a fixed crate name) so that
/// running/building a specific `--bin` never has to pull in the rest of the
/// workspace's dependency graph — in particular, xtask's own `std`-only
/// deps, which can't compile for the `no_std` ESP32 target.
pub fn package_owning_bin(bin: &str) -> Result<String> {
    let cargo = env::var("CARGO").unwrap_or_else(|_| "cargo".into());

    let output = Command::new(&cargo)
        .args(["metadata", "--no-deps", "--format-version=1"])
        .output()
        .context("failed to run `cargo metadata`")?;

    if !output.status.success() {
        bail!(
            "`cargo metadata` failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let metadata: WorkspaceMetadata = serde_json::from_slice(&output.stdout)
        .context("failed to parse `cargo metadata` output")?;

    for package in &metadata.packages {
        for target in &package.targets {
            if target.name == bin && target.kind.iter().any(|kind| kind == "bin") {
                return Ok(package.name.clone());
            }
        }
    }

    bail!("no binary target named `{bin}` found in this workspace")
}
