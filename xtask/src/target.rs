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
}

impl fmt::Display for Target {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Target::Local => f.write_str("local"),
            Target::Esp32 => f.write_str("esp32"),
        }
    }
}
