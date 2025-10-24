#![no_std]
#![cfg(target_os = "none")]

use serde::{Deserialize, Serialize};

extern crate alloc;

cfg_if::cfg_if! {
    if #[cfg(feature = "ember-based")] {
        pub mod control;
        pub mod light;
        pub mod moist;
        pub mod pump;
        pub mod temp;

        mod notif;
    } else {
        pub mod without_ember;
    }
}

const LIGHT_ALERT_THRESHOLD: f32 = 200.0;
const LIGHT_LOW_THRESHOLD: f32 = 100.0;
const LIGHT_HIGH_THRESHOLD: f32 = 2200.0;

const MOISTURE_THRESHOLD: f32 = 60.0;
const MOISTURE_LOW_THRESHOLD: f32 = 30.0;
const MOISTURE_HIGH_THRESHOLD: f32 = 80.0;

// const TEMP_HIGH_THRESHOLD: f32 = 36.0;
// const TEMP_LOW_THRESHOLD: f32 = -18.0;

const LDR_RL10: f32 = 39.0;
const LDR_GAMMA: f32 = 0.5;
const LDR_VCC_VOLTAGE: f32 = 3.3;

const LDR_ADC_RANGE_OFFSET: f32 = -500.0;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Measurement {
    pub temperature: f32,
    pub humidity: f32,
}
