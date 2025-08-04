#![no_std]
#![cfg(target_os = "none")]

extern crate alloc;

use core::cell::OnceCell;

use blocking_network_stack::Stack;
use esp_wifi::wifi::{WifiDevice, WifiStaDevice};

const MOISTURE_THRESHOLD: f32 = 60.0;
const FAN_TEMPERATURE_THRESHOLD: f32 = -1.0;

const TEMP_SENSOR_VCC_VOLTAGE: f32 = 3.3;

const HTTP_SERVER_PORT: u16 = 80;

pub static mut WIFI_STACK: OnceCell<Stack<'static, WifiDevice<'static, WifiStaDevice>>> =
    OnceCell::new();

pub mod control;
pub mod temp;

mod utils;
