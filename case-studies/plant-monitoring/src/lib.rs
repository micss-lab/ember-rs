#![no_std]
#![cfg(target_os = "none")]

extern crate alloc;

pub mod control;
pub mod light;
pub mod moist;
pub mod pump;
pub mod temp;

mod notif;
mod util;

const LIGHT_ALERT_THRESHOLD: f32 = 2000.0;
const LIGHT_LOW_THRESHOLD: f32 = 100.0;
const LIGHT_HIGH_THRESHOLD: f32 = 2200.0;

const MOISTURE_THRESHOLD: f32 = 60.0;
const MOISTURE_LOW_THRESHOLD: f32 = 30.0;
const MOISTURE_HIGH_THRESHOLD: f32 = 80.0;

// const TEMP_HIGH_THRESHOLD: f32 = 36.0;
// const TEMP_LOW_THRESHOLD: f32 = -18.0;

const MIN_LUX: f32 = 0.1;
const MAX_LUX: f32 = 100000.0;
