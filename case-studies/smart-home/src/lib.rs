#![no_std]
#![cfg(target_os = "none")]

extern crate alloc;

const MOISTURE_THRESHOLD: f32 = 60.0;
const FAN_TEMPERATURE_THRESHOLD: f32 = -1.0;

const TEMP_SENSOR_VCC_VOLTAGE: f32 = 3.3;

pub mod control;
pub mod temp;

mod utils;
