use anyhow::Result;

use crate::cargo::run_cargo;
use crate::target::Target;

pub fn run(target: Option<Target>) -> Result<()> {
    let targets: Vec<Target> = match target {
        Some(target) => vec![target],
        None => Target::ALL.to_vec(),
    };

    for target in targets {
        eprintln!("\n=== check: {target} ===");

        let mut args: Vec<&str> = vec!["check"];
        args.extend(target.check_like_args());
        run_cargo(&args)?;
    }

    Ok(())
}
