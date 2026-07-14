use anyhow::Result;

use crate::cargo::run_cargo;
use crate::target::Target;

/// Tests only run on the local target: the ESP32 target has no test harness
/// and would require real hardware in the loop.
pub fn run() -> Result<()> {
    let target = Target::Local;
    run_cargo(&["test", "--workspace", "--target", target.triple()])
}
