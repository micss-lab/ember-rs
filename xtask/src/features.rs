use std::collections::HashMap;
use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use serde::Deserialize;

use crate::target::Target;

/// Which target(s) a feature is valid/relevant for, as declared in
/// `features.toml`.
#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "lowercase")]
enum Scope {
    Local,
    Esp32,
    Both,
}

impl Scope {
    fn matches(self, target: Target) -> bool {
        matches!(
            (self, target),
            (Scope::Both, _) | (Scope::Local, Target::Local) | (Scope::Esp32, Target::Esp32)
        )
    }
}

/// crate name -> feature name -> scope
type Config = HashMap<String, HashMap<String, Scope>>;

/// Loads `xtask/features.toml`, resolved relative to xtask's own manifest
/// directory so it works regardless of the caller's current working
/// directory. Read fresh on every call so editing the file doesn't require
/// recompiling xtask.
fn load() -> Result<Config> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("features.toml");
    let raw =
        fs::read_to_string(&path).with_context(|| format!("failed to read {}", path.display()))?;
    toml::from_str(&raw).with_context(|| format!("failed to parse {}", path.display()))
}

/// Per-crate bare feature names valid for `target`, one entry per crate that
/// has at least one such feature (crates with none are omitted).
///
/// Kept per-crate, rather than unioned into one flat list, because
/// `cargo-hack`'s `--ignore-unknown-features` doesn't reliably suppress an
/// error when a feature name from one crate's list is passed to a different
/// crate that doesn't declare it — so each crate must be hacked in isolation
/// (`-p <crate> --include-features <that crate's own names>`) rather than as
/// one `--workspace --include-features <everyone's names>` run.
pub fn crate_feature_names_for(target: Target) -> Result<Vec<(String, Vec<String>)>> {
    let config = load()?;
    let mut result: Vec<(String, Vec<String>)> = config
        .into_iter()
        .filter_map(|(krate, features)| {
            let mut names: Vec<String> = features
                .into_iter()
                .filter(|(_, scope)| scope.matches(target))
                .map(|(name, _)| name)
                .collect();
            names.sort();
            (!names.is_empty()).then_some((krate, names))
        })
        .collect();
    result.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(result)
}

/// `"<crate>/<feature>"` strings valid for `target`, for a plain
/// `cargo clippy --features ...` invocation, which needs workspace-qualified
/// feature paths.
pub fn qualified_features_for(target: Target) -> Result<Vec<String>> {
    let config = load()?;
    let mut qualified: Vec<String> = config
        .iter()
        .flat_map(|(krate, features)| {
            features
                .iter()
                .filter(|(_, scope)| scope.matches(target))
                .map(move |(feature, _)| format!("{krate}/{feature}"))
        })
        .collect();
    qualified.sort();
    Ok(qualified)
}
