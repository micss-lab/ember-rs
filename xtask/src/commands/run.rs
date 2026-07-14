use anyhow::Result;

use crate::cargo::run_cargo;
use crate::metadata;
use crate::target::Target;

pub fn run(target: Target, bin: &str, extra: &[String]) -> Result<()> {
    // `cargo run` has no `--workspace`/`--exclude`, so without an explicit
    // `-p`, cargo has to consider the whole workspace's graph to disambiguate
    // the bin — including xtask's own std-only deps, which can't compile
    // under `-Zbuild-std` for the ESP32 target. Resolving the owning package
    // up front keeps the build scoped to just that package.
    let package = metadata::package_owning_bin(bin)?;

    let mut args: Vec<&str> = vec!["run", "--target", target.triple()];
    args.extend_from_slice(target.extra_args());
    args.push("-p");
    args.push(&package);
    args.push("--bin");
    args.push(bin);
    args.extend(extra.iter().map(String::as_str));

    run_cargo(&args)
}
