use anyhow::Result;

use crate::cargo::run_cargo;
use crate::target::Target;

pub fn run(target: Option<Target>, fix: bool) -> Result<()> {
    let targets: Vec<Target> = match target {
        Some(target) => vec![target],
        None => Target::ALL.to_vec(),
    };

    for target in targets {
        eprintln!("\n=== clippy: {target} ===");

        let mut args: Vec<&str> = vec!["clippy", "--workspace", "--target", target.triple()];
        if let Some(exclude) = target.workspace_exclude() {
            args.push("--exclude");
            args.push(exclude);
        }
        args.extend_from_slice(target.extra_args());
        if fix {
            args.push("--fix");
            args.push("--allow-dirty");

            // `--fix` unconditionally implies `--all-targets`, which tries to
            // build the `tests` target for every crate. That's impossible on
            // ESP32: these crates are `#![no_std]`/`#![no_main]` with no
            // custom test harness, so there's no `test` crate, allocator, or
            // panic handler available to link a test binary against.
            // Explicit `--lib --bins` overrides the implied `--all-targets`
            // and covers everything that's actually buildable there (library
            // crates' `lib` target, and the examples crate's `bin` targets).
            if target == Target::Esp32 {
                args.push("--lib");
                args.push("--bins");
            }
        }
        run_cargo(&args)?;
    }

    Ok(())
}
