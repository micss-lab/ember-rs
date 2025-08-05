#![no_std]
#![cfg(target_os = "none")]

extern crate alloc;

use core::cell::OnceCell;

use blocking_network_stack::Stack;
use esp_wifi::{
    wifi::{WifiDevice, WifiStaDevice},
    EspWifiController,
};
use smoltcp::iface::SocketStorage;

const SSID: Option<&str> = option_env!("CASE_STUDY_SSID");
const AP_PASSWORD: Option<&str> = option_env!("CASE_STUDY_AP_PASSWORD");
const WIFI_CHANNEL: Option<u8> = Some(6);
const WIFI_AP_SCAN_COUNT: u32 = 3;

const SOCKET_COUNT: usize = 10;

pub static mut SOCKET_STORE: [SocketStorage; SOCKET_COUNT] = [SocketStorage::EMPTY; SOCKET_COUNT];
pub static mut WIFI_INIT: OnceCell<EspWifiController> = OnceCell::new();
pub static mut WIFI_STACK: OnceCell<Stack<'static, WifiDevice<'static, WifiStaDevice>>> =
    OnceCell::new();

// ====== Configuration values. ======

const MOISTURE_THRESHOLD: f32 = 60.0;
const FAN_TEMPERATURE_THRESHOLD: f32 = -1.0;

const TEMP_SENSOR_VCC_VOLTAGE: f32 = 3.3;

const HTTP_SERVER_PORT: u16 = 80;

pub mod entry;
pub mod wifi;

mod control;
mod discovery;
mod temp;
mod utils;
