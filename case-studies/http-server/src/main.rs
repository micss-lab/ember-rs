#![cfg_attr(target_os = "none", no_std)]
#![cfg_attr(target_os = "none", no_main)]

#[cfg(target_os = "none")]
extern crate alloc;

#[cfg(not(target_os = "none"))]
fn main() {
    panic!("Use cases can only be run on esp32 based targets");
}

#[cfg(target_os = "none")]
mod entry;

#[cfg(target_os = "none")]
#[esp_hal::entry]
fn main() -> ! {
    entry::main();
    panic!("End of program.")
}
