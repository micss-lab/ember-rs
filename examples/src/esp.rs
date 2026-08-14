use esp_hal::ram;

const HEAP_SIZE: usize = 128 * 1024;
// Second heap region, reclaimed from the esp-idf bootloader's own scratch
// space (`dram2_seg` in esp-hal's linker script, ~96.4KB total. Needs the
// `__esp_idf_bootloader` feature on `esp-hal-procmacros` (forced on via
// examples/Cargo.toml, since esp-hal itself doesn't forward it) and the
// esp-idf 2nd-stage bootloader, which this project already uses.
const RECLAIMED_HEAP_SIZE: usize = 90 * 1024;

pub fn init_heap() {
    esp_alloc::heap_allocator!(size: HEAP_SIZE);
    esp_alloc::heap_allocator!(#[ram(reclaimed)] size: RECLAIMED_HEAP_SIZE);
}
