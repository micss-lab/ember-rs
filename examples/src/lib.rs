#![cfg_attr(target_os = "none", no_std)]

#[cfg(target_os = "none")]
pub mod esp;

#[cfg(not(target_os = "none"))]
pub mod local;

mod setup_example {
    #[macro_export]
    macro_rules! setup_example {
        () => {
            extern crate alloc;

            #[cfg(target_os = "none")]
            mod esp_imports {
                pub(super) use esp_backtrace as _;
                pub(super) use esp_hal_embassy as _;
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
                esp::init_heap();

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
