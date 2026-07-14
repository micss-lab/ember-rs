use anyhow::Result;

use crate::cargo::run_cargo;
use crate::features;
use crate::target::Target;

/// Emits `cargo clippy --message-format=json` diagnostics for both targets,
/// concatenated on stdout, for use as `rust-analyzer.check.overrideCommand`.
/// rust-analyzer can only run one process, but this workspace has two
/// mutually-incompatible targets (std vs. no_std), so we run clippy once per
/// target here and let both JSON-lines streams flow through.
pub fn run() -> Result<()> {
    for target in Target::ALL {
        let features = features::qualified_features_for(target)?.join(",");

        let mut args: Vec<&str> = vec![
            "clippy",
            "--workspace",
            "--target",
            target.triple(),
            "--message-format=json",
        ];
        if let Some(exclude) = target.workspace_exclude() {
            args.push("--exclude");
            args.push(exclude);
        }
        args.extend_from_slice(target.extra_args());
        if !features.is_empty() {
            args.push("--features");
            args.push(&features);
        }

        // A compile error in one target's diagnostics still gets flushed to
        // stdout before cargo exits non-zero; don't let that suppress the
        // other target's diagnostics or xtask's own exit code.
        if let Err(err) = run_cargo(&args) {
            log::warn!("{err:#}");
        }
    }

    Ok(())
}
