use std::{
    borrow::Cow,
    collections::HashSet,
    process::{Command, Stdio},
};

use anyhow::{Context, Result, bail};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Target {
    Linux,
    Esp32,
}

impl Target {
    fn into_args(self) -> Vec<Cow<'static, str>> {
        match self {
            Target::Linux => Vec::from(["--target".into(), "x86_64-unknown-linux-gnu".into()]),
            Target::Esp32 => Vec::from([
                "--target".into(),
                "xtensa-esp32-none-elf".into(),
                "-Zbuild-std=core,alloc".into(),
            ]),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Cargo {
    targets: HashSet<Target>,
}

impl Default for Cargo {
    fn default() -> Self {
        Self {
            targets: HashSet::from([Target::Linux, Target::Esp32]),
        }
    }
}

impl Cargo {
    pub fn build(self) -> Result<()> {
        self.exec_for_every_target(CargoCommand::Build)
    }

    pub fn check(self) -> Result<()> {
        self.exec_for_every_target(CargoCommand::Check)
    }

    fn exec_for_every_target(self, command: CargoCommand) -> Result<()> {
        for target in self.targets {
            exec(command, Options::new(Some(target)), Vec::with_capacity(0))?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
enum CargoCommand {
    Build,
    Check,
}

impl std::fmt::Display for CargoCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Build => "build",
            Self::Check => "check",
        })
    }
}

#[derive(Debug, Clone)]
struct Options {
    target: Option<Target>,
    release: bool,
    exclude: Vec<&'static str>,
}

impl Options {
    fn new(target: Option<Target>) -> Self {
        Self {
            target,
            release: true,
            exclude: Vec::from(["xtask"]),
        }
    }
}

impl Options {
    fn into_vec(self) -> Vec<Cow<'static, str>> {
        let mut options = Vec::new();
        if let Some(target) = self.target {
            options.extend(target.into_args());
        }

        if self.release {
            options.push("--release".into());
        }

        if !self.exclude.is_empty() {
            options.push("--workspace".into());
            options.extend(["--exclude".into(), self.exclude.join(",").into()]);
        }
        options
    }

    fn to_vec(&self) -> Vec<Cow<'static, str>> {
        self.clone().into_vec()
    }
}

impl std::fmt::Display for Options {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_vec().join(" "))
    }
}

fn exec(
    command: CargoCommand,
    options: Options,
    args: impl AsRef<[Cow<'static, str>]>,
) -> Result<()> {
    let args = args.as_ref();
    log::debug!(
        "Executing command `cargo {command} {options} {}`",
        args.join(" ")
    );

    let output = Command::new(
        std::env::var("CARGO").context("getting cargo executable from the environment")?,
    )
    .arg(command.to_string())
    .args(options.into_vec().into_iter().map(Cow::into_owned))
    .args(args.iter().cloned().map(Cow::into_owned))
    .stdout(Stdio::inherit())
    .stderr(Stdio::inherit())
    .output()
    .context("executing cargo command")?;

    if output.status.success() {
        Ok(())
    } else {
        bail!("cargo command exited with non-zero exit code");
    }
}
