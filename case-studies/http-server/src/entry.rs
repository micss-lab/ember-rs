extern crate alloc;

use esp_backtrace as _;

const HEAP_SIZE: usize = 32 * 1024;

pub(crate) fn main() {
    // Set newline mode to linux line endings.
    esp_println::print!("\x1b[20h");
    esp_println::logger::init_logger_from_env();
    esp_alloc::heap_allocator!(HEAP_SIZE);
}
