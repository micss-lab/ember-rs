#![no_std]
#![cfg(target_os = "none")]

use esp_hal::{Blocking, uart::UartRx};

extern crate alloc;

cfg_if::cfg_if! {
    if #[cfg(feature = "ember-based")] {
        pub mod control;
        pub mod dht22;
        pub mod fan;
        pub mod lock;
        pub mod pir;
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Measurement {
    pub temperature: f32,
    pub humidity: f32,
}

pub fn read_chars_from_uart(buffer: &mut [u8], serial_rx: &mut UartRx<'static, Blocking>) -> usize {
    let mut read_chars = 0;
    while read_chars != buffer.len() {
        let mut buf = [0u8; 1];
        let byte = match serial_rx.read_buffered_bytes(&mut buf) {
            Ok(0) => continue,
            Ok(1) => {
                let b = buf[0];
                log::debug!("byte: {b}");
                b
            }
            Ok(_) => unreachable!("cannot read more bytes than size of buffer"),
            Err(e) => panic!("failed to read from console: {:?}", e),
        };

        if byte == b'\n' || byte == b'\r' {
            break;
        }
        buffer[read_chars] = byte;
        read_chars += 1;
        if read_chars == 25 {
            break;
        }
    }
    read_chars
}
