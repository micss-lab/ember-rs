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
        // the doc comment on `features::crate_feature_names_excluded_for`
        // for why, and for why `--exclude-features` is used over
        // `--include-features`.
        for (krate, excluded_features) in features::crate_feature_names_excluded_for(target)? {
            let exclude = excluded_features.join(",");
            let group = target.hack_feature_group(&krate);
            let group_arg = group.map(|(existing, extra)| format!("{existing},{extra}"));

            let mut args: Vec<&str> =
                vec!["hack", "check", "-p", &krate, "--target", target.triple()];
            args.extend_from_slice(target.extra_args());
            args.push("--feature-powerset");
            if !exclude.is_empty() {
                args.push("--exclude-features");
                args.push(&exclude);
            }
            if let Some(group_arg) = &group_arg {
                args.push("--group-features");
                args.push(group_arg);
            }

            run_cargo(&args)?;
        }
    }

    Ok(())
}
