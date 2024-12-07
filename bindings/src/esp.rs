use core::panic::PanicInfo;

use esp_alloc as _;

#[panic_handler]
fn panic(_: &PanicInfo) -> ! {
    loop {}
}
