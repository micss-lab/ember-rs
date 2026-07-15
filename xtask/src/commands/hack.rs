use std::process::Command;

use anyhow::{Result, bail};

use crate::cargo::run_cargo;
use crate::features;
use crate::target::Target;

fn ensure_installed() -> Result<()> {
    let installed = Command::new("cargo")
        .args(["hack", "--version"])
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false);

    if !installed {
        bail!("cargo-hack is not installed; run `cargo install cargo-hack` first");
    }

    Ok(())
}

pub fn run(target: Option<Target>) -> Result<()> {
    ensure_installed()?;

    let targets: Vec<Target> = match target {
        Some(target) => vec![target],
        None => Target::ALL.to_vec(),
    };

    for target in targets {
        eprintln!("\n=== hack: {target} ===");

        // Each crate is hacked in isolation (`-p`), not `--workspace`: see
        // the doc comment on `features::crate_feature_names_for` for why.
        for (krate, features) in features::crate_feature_names_for(target)? {
            let include = features.join(",");

            let mut args: Vec<&str> =
                vec!["hack", "check", "-p", &krate, "--target", target.triple()];
            args.extend_from_slice(target.extra_args());
            args.push("--feature-powerset");
            args.push("--include-features");
            args.push(&include);

            run_cargo(&args)?;
        }
    }

    Ok(())
}
