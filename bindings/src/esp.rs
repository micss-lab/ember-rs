use core::panic::PanicInfo;

use esp_alloc as _;
use esp_println::logger::init_logger;

const HEAP_SIZE: usize = 160 * 1024;

#[panic_handler]
fn panic(_: &PanicInfo) -> ! {
    log::error!("Got panic!\r");
    loop {}
}

pub(crate) fn initialize_logging(level: log::LevelFilter) {
    init_logger(level)
}

pub(crate) fn initialize_allocator() {
    esp_alloc::heap_allocator!(HEAP_SIZE);
}

/// Esp32 single core critical section implementation.
///
/// Stolen from the [esp-hal] crate.
///
/// [esp-hal]: https://github.com/esp-rs/esp-hal/blob/d2f15d69d7004a5360e7d4ab9311a6f6ac069337/esp-hal/src/sync.rs
mod critical_section {
    use core::cell::Cell;

    struct CriticalSection;

    critical_section::set_impl!(CriticalSection);

    static CRITICAL_SECTION: Lock = Lock::new();

    unsafe impl critical_section::Impl for CriticalSection {
        unsafe fn acquire() -> critical_section::RawRestoreState {
            CRITICAL_SECTION.acquire()
        }

        unsafe fn release(token: critical_section::RawRestoreState) {
            CRITICAL_SECTION.release(token);
        }
    }

    /// A lock that can be used to protect shared resources.
    struct Lock {
        is_locked: Cell<bool>,
    }

    unsafe impl Sync for Lock {}

    impl Default for Lock {
        fn default() -> Self {
            Self::new()
        }
    }

    // The restore state is a u8 that is casted from a bool, so it has a value of
    // 0x00 or 0x01 before we add the reentry flag to it.
    const REENTRY_FLAG: u8 = 1 << 7;

    impl Lock {
        /// Create a new lock.
        const fn new() -> Self {
            Self {
                is_locked: Cell::new(false),
            }
        }

        /// Acquires the lock.
        ///
        /// # Safety
        ///
        /// - Each release call must be paired with an acquire call.
        /// - The returned token must be passed to the corresponding `release` call.
        /// - The caller must ensure to release the locks in the reverse order they
        ///   were acquired.
        unsafe fn acquire(&self) -> critical_section::RawRestoreState {
            let mut tkn = unsafe { single_core::disable_interrupts() };
            let was_locked = self.is_locked.replace(true);
            if was_locked {
                tkn |= REENTRY_FLAG;
            }
            tkn
        }

        /// Releases the lock.
        ///
        /// # Safety
        ///
        /// - This function must only be called if the lock was acquired by the
        ///   current thread.
        /// - The caller must ensure to release the locks in the reverse order they
        ///   were acquired.
        /// - Each release call must be paired with an acquire call.
        unsafe fn release(&self, token: critical_section::RawRestoreState) {
            if token & REENTRY_FLAG == 0 {
                self.is_locked.set(false);

                single_core::reenable_interrupts(token);
            }
        }
    }

    mod single_core {
        use core::sync::atomic::{compiler_fence, Ordering};

        pub(super) unsafe fn disable_interrupts() -> critical_section::RawRestoreState {
            let mut mstatus = 0u32;
            core::arch::asm!("csrrci {0}, mstatus, 8", inout(reg) mstatus);
            let token = ((mstatus & 0b1000) != 0) as critical_section::RawRestoreState;

            // Ensure no subsequent memory accesses are reordered to before interrupts are
            // disabled.
            compiler_fence(Ordering::SeqCst);

            token
        }

        pub(super) unsafe fn reenable_interrupts(token: critical_section::RawRestoreState) {
            // Ensure no preceeding memory accesses are reordered to after interrupts are
            // enabled.
            compiler_fence(Ordering::SeqCst);

            if token != 0 {
                esp_riscv_rt::riscv::interrupt::enable();
            }
        }
    }
}
