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

        let mut args: Vec<&str> = vec!["check", "--workspace", "--target", target.triple()];
        if let Some(exclude) = target.workspace_exclude() {
            args.push("--exclude");
            args.push(exclude);
        }
        // Only the local target can check `#[cfg(test)]` code: it's the only one with a real
        // `std`, which the `#[test]` harness needs. ESP32 is a freestanding `-none-elf` target
        // built with `-Zbuild-std=core,alloc` only; `std` (and therefore `test`) cannot be
        // built for it at all (its global allocator needs OS-level realloc/alloc_zeroed).
        if target == Target::Local {
            args.push("--all-targets");
        }
        args.extend_from_slice(target.extra_args());
        run_cargo(&args)?;
    }

    Ok(())
}
