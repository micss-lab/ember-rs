#![cfg_attr(target_os = "none", no_std)]

#[cfg(target_os = "none")]
pub mod esp;

#[cfg(not(target_os = "none"))]
pub mod local;

mod setup_example {
    #[macro_export]
    macro_rules! setup_example {
        () => {
            $crate::setup_example!(heap_size: $crate::esp::DEFAULT_HEAP_SIZE);
        };
        // `heap_size` trims the *primary* heap only (see `esp::
        // RECLAIMED_HEAP_SIZE`'s doc comment for why): a binary whose
        // ESP-NOW/WiFi receive processing needs more headroom on the main
        // task's stack, which shares memory with this heap, passes a
        // smaller size here instead of using the shared default every
        // binary otherwise gets.
        //
        // Expands both `heap_allocator!` calls inline, rather than calling
        // a shared function with `$heap_size`: `heap_allocator!` sizes a
        // `static` from its `size:` argument, and a `static` can't
        // reference a generic parameter from an enclosing function
        // (E0401), so this has to happen at each call site, not once in
        // `ember_examples::esp`.
        (heap_size: $heap_size:expr) => {
            extern crate alloc;

            #[cfg(target_os = "none")]
            mod esp_imports {
                pub(super) use esp_backtrace as _;
                pub(super) use esp_println::print;

                pub(super) use ember_examples::esp;
            }

            #[cfg(target_os = "none")]
            use esp_imports::*;

            #[cfg(target_os = "none")]
            #[esp_hal::main]
            fn main() -> ! {
                // Set newline mode to linux line endings.
                print!("\x1b[20h");
                esp_println::logger::init_logger_from_env();
                esp_alloc::heap_allocator!(size: $heap_size);
                esp_alloc::heap_allocator!(#[esp_hal::ram(reclaimed)] size: esp::RECLAIMED_HEAP_SIZE);

                example();

                panic!("End of program");
            }

            #[cfg(not(target_os = "none"))]
            fn main() {
                use ember_examples::local;
                local::init_logger(log::LevelFilter::Trace);

                example();
            }
        };
    }
}
