use anyhow::Result;

use crate::cargo::run_cargo;
use crate::features;
use crate::target::Target;

/// Emits `cargo clippy --message-format=json` diagnostics for both targets,
/// concatenated on stdout, for use as `rust-analyzer.check.overrideCommand`.
/// rust-analyzer can only run one process, but this workspace has two
/// mutually-incompatible targets (std vs. no_std), so we run clippy once per
/// target here and let both JSON-lines streams flow through.
///
/// `color` requests `json-diagnostic-rendered-ansi`, whose `rendered` field
/// carries ANSI escapes: consumers that display the rendered text as-is
/// (e.g. bacon) want this, but rust-analyzer doesn't unless the client
/// declares the `colorDiagnosticOutput` experimental capability, so it stays
/// off there.
pub fn run(color: bool) -> Result<()> {
    let message_format = if color {
        "--message-format=json,json-diagnostic-rendered-ansi"
    } else {
        "--message-format=json"
    };

    for target in Target::ALL {
        let features = features::qualified_features_for(target)?.join(",");

        let mut args: Vec<&str> = vec!["clippy"];
        args.extend(target.check_like_args());
        args.push(message_format);
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
