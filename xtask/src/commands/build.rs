use anyhow::Result;

use crate::cargo::run_cargo;
use crate::metadata;
use crate::target::Target;

pub fn run(target: Option<Target>, bin: Option<String>, extra: &[String]) -> Result<()> {
    let targets: Vec<Target> = match target {
        Some(target) => vec![target],
        None => Target::ALL.to_vec(),
    };

    // See the comment in commands::run for why `--bin` needs an explicit
    // `-p` rather than relying on cargo to disambiguate across the whole
    // workspace.
    let package = bin
        .as_deref()
        .map(metadata::package_owning_bin)
        .transpose()?;

    let multi_target = targets.len() > 1;

    for target in targets {
        if multi_target {
            eprintln!("\n=== build: {target} ===");
        }

        let mut args: Vec<&str> = vec!["build", "--target", target.triple()];
        args.extend_from_slice(target.extra_args());

        match (&package, &bin) {
            (Some(package), Some(bin)) => {
                args.push("-p");
                args.push(package);
                args.push("--bin");
                args.push(bin);
            }
            _ => {
                args.push("--workspace");
                if let Some(exclude) = target.workspace_exclude() {
                    args.push("--exclude");
                    args.push(exclude);
                }
            }
        }

        args.extend(extra.iter().map(String::as_str));

        run_cargo(&args)?;
    }

    Ok(())
}
