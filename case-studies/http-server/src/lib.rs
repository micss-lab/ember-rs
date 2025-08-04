#![no_std]
#![cfg(target_os = "none")]

use core::cell::OnceCell;

use blocking_network_stack::Stack;
use esp_wifi::wifi::{WifiDevice, WifiStaDevice};

extern crate alloc;

pub static mut WIFI_STACK: OnceCell<Stack<'static, WifiDevice<'static, WifiStaDevice>>> =
    OnceCell::new();

pub mod http;
