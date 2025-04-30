use cfg_if::cfg_if;
use esp_println::print;
use log::LevelFilter;

/// Intializes the libraries global logger.
///
/// This function is idempotent. Logging will only be initialized and configured the first time it
/// is called.
pub(crate) fn initialize_logging(level: LevelFilter) {
    static mut INIT: bool = false;

    critical_section::with(|_| {
        if unsafe { INIT } {
            return;
        }

        cfg_if! {
            if #[cfg(target_os = "none")] {
                use crate::esp;
                esp::initialize_logging(level);
            } else {
                // Do nothing for now.
                let _ = level;
            }
        }

        // Set newline mode to linux line endings.
        print!("\x1b[20h");

        unsafe { INIT = true };
    })
}
