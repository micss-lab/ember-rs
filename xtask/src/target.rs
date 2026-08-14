use std::fmt;

/// The two build targets ember-rs cares about: the host (used for local
/// development/testing) and the ESP32 (the framework's embedded target).
#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum Target {
    #[value(name = "local")]
    Local,
    #[value(name = "esp32")]
    Esp32,
}

impl Target {
    pub const ALL: [Target; 2] = [Target::Local, Target::Esp32];

    /// The rustc target triple for this target.
    pub fn triple(self) -> &'static str {
        match self {
            Target::Local => "x86_64-unknown-linux-gnu",
            Target::Esp32 => "xtensa-esp32-none-elf",
        }
    }

    /// Extra cargo flags required to build for this target.
    pub fn extra_args(self) -> &'static [&'static str] {
        match self {
            Target::Local => &[],
            Target::Esp32 => &["-Zbuild-std=core,alloc"],
        }
    }

    /// Package to exclude from `--workspace` invocations, if any. xtask
    /// itself depends on `std` (via clap/anyhow/env_logger) and cannot be
    /// built for a `no_std` target.
    pub fn workspace_exclude(self) -> Option<&'static str> {
        match self {
            Target::Local => None,
            Target::Esp32 => Some("xtask"),
        }
    }

    /// For `xtask hack` on this target: an extra feature to fold into
    /// `krate`'s feature-powerset, grouped 1:1 with an existing feature of
    /// that powerset via `cargo hack --group-features` (so combos get both
    /// or neither, never one without the other). The existing feature only
    /// compiles when the extra one is also on, and nothing in an isolated
    /// `-p` check supplies it the way the binary crate normally would via
    /// feature unification. See `ember-acc/Cargo.toml`'s `rt` feature.
    /// Returns `(existing_feature, extra_feature)`.
    pub fn hack_feature_group(self, krate: &str) -> Option<(&'static str, &'static str)> {
        match (self, krate) {
            (Target::Esp32, "ember-acc") => Some(("espnow", "rt")),
            (Target::Esp32, "ember") => Some(("acc-espnow", "ember-acc/rt")),
            _ => None,
        }
    }
}

impl fmt::Display for Target {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Target::Local => f.write_str("local"),
            Target::Esp32 => f.write_str("esp32"),
        }
    }
}
