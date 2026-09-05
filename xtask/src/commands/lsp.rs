use std::collections::HashSet;
use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};

use anyhow::{Context, Result};

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

    // `Target::Local`'s `--all-targets` compiles the lib once per cargo
    // target kind (lib/bins/tests/examples), so the same diagnostic comes
    // back once per kind. Dedupe across the whole run (both cargo targets)
    // by the diagnostic's rendered text, which is identical across kinds.
    let mut seen = HashSet::new();

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
        if let Err(err) = run_cargo_deduped(&args, &mut seen) {
            log::warn!("{err:#}");
        }
    }

    Ok(())
}

/// Like `cargo::run_cargo`, but captures stdout to drop compiler-message
/// lines already seen (by rendered text) in an earlier call, then re-emits
/// the rest on this process's stdout. Stderr is inherited as usual.
fn run_cargo_deduped(args: &[&str], seen: &mut HashSet<String>) -> Result<()> {
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".into());
    let joined = args.join(" ");

    eprintln!("$ {cargo} {joined}");

    let mut child = Command::new(&cargo)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .with_context(|| format!("failed to execute `{cargo} {joined}`"))?;

    let stdout = child.stdout.take().expect("stdout was piped");
    let mut out = std::io::stdout().lock();
    for line in BufReader::new(stdout).lines() {
        let line = line.context("failed to read cargo stdout")?;
        if is_duplicate_compiler_message(&line, seen) {
            continue;
        }
        writeln!(out, "{line}").context("failed to write to stdout")?;
    }

    let status = child.wait().context("failed to wait on cargo")?;
    if !status.success() {
        anyhow::bail!("`{cargo} {joined}` exited with {status}");
    }

    Ok(())
}

/// Returns `true` if `line` is a `compiler-message` whose rendered text was
/// already seen. Non-JSON lines and other reasons (`build-finished`, etc.)
/// are never considered duplicates.
fn is_duplicate_compiler_message(line: &str, seen: &mut HashSet<String>) -> bool {
    let Ok(value) = serde_json::from_str::<serde_json::Value>(line) else {
        return false;
    };
    if value.get("reason").and_then(|r| r.as_str()) != Some("compiler-message") {
        return false;
    }
    let Some(rendered) = value.pointer("/message/rendered").and_then(|r| r.as_str()) else {
        return false;
    };
    !seen.insert(rendered.to_string())
}
