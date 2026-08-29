pub const DEFAULT_HEAP_SIZE: usize = 128 * 1024;
// Second heap region, reclaimed from the esp-idf bootloader's own scratch
// space (`dram2_seg` in esp-hal's linker script, ~96.4KB total. Needs the
// `__esp_idf_bootloader` feature on `esp-hal-procmacros` (forced on via
// examples/Cargo.toml, since esp-hal itself doesn't forward it) and the
// esp-idf 2nd-stage bootloader, which this project already uses. Doesn't
// compete with `DEFAULT_HEAP_SIZE`/the stack for space, unlike the primary
// heap, so there's no reason for callers to tune this one.
pub const RECLAIMED_HEAP_SIZE: usize = 90 * 1024;

// No `init_heap()` function here: `esp_alloc::heap_allocator!` sizes a
// `static` from its `size:` argument, and a `static` can't reference a
// generic parameter from an enclosing function (E0401) - passing the size
// through a function call, generic or not, doesn't work. `setup_example!`
// (`lib.rs`) instead expands both `heap_allocator!` calls directly inline,
// using `DEFAULT_HEAP_SIZE`/`RECLAIMED_HEAP_SIZE` from here or a caller's
// own override.
