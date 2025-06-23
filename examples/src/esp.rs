const HEAP_SIZE: usize = 72 * 1024;

pub fn init_heap() {
    esp_alloc::heap_allocator!(HEAP_SIZE);
}
