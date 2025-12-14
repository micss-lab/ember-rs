use core::panic::PanicInfo;

use esp_println::logger::init_logger;

mod alloc;
mod critical_section;

#[panic_handler]
fn panic(p: &PanicInfo) -> ! {
    log::error!("Got panic!");
    log::error!("{}", p);
    loop {}
}

pub(crate) fn initialize_logging(level: log::LevelFilter) {
    init_logger(level)
}
