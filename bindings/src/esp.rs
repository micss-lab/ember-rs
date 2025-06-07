use core::panic::PanicInfo;

use esp_alloc as _;
use esp_println::logger::init_logger;

const HEAP_SIZE: usize = 160 * 1024;

#[panic_handler]
fn panic(_: &PanicInfo) -> ! {
    log::error!("Got panic!");
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
        inner: multicore::AtomicLock,
    }

    unsafe impl Sync for Lock {}

    impl Default for Lock {
        fn default() -> Self {
            Self::new()
        }
    }

    // PS has 15 useful bits. Bits 12..16 and 19..32 are unused, so we can use bit
    // #31 as our reentry flag.
    // We can assume the reserved bit is 0 otherwise rsil - wsr pairings would be
    // undefined behavior: Quoting the ISA summary, table 64:
    // Writing a non-zero value to these fields results in undefined processor behavior.
    const REENTRY_FLAG: u32 = 1 << 31;

    impl Lock {
        /// Create a new lock.
        const fn new() -> Self {
            Self {
                inner: multicore::AtomicLock::new(),
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
            // We acquire the lock inside an interrupt-free context to prevent a subtle
            // race condition:
            // In case an interrupt handler tries to lock the same resource, it could win if
            // the current thread is holding the lock but isn't yet in interrupt-free context.
            // If we maintain non-reentrant semantics, this situation would panic.
            // If we allow reentrancy, the interrupt handler would technically be a different
            // context with the same `current_thread_id`, so it would be allowed to lock the
            // resource in a theoretically incorrect way.
            let try_lock = |current_thread_id| {
                let mut tkn = unsafe { single_core::disable_interrupts() };

                match self.inner.try_lock(current_thread_id) {
                    Ok(()) => Some(tkn),
                    Err(owner) if owner == current_thread_id => {
                        tkn |= REENTRY_FLAG;
                        Some(tkn)
                    }
                    Err(_) => {
                        unsafe { single_core::reenable_interrupts(tkn) };
                        None
                    }
                }
            };

            let current_thread_id = multicore::thread_id();
            loop {
                if let Some(token) = try_lock(current_thread_id) {
                    return token;
                }
            }
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
                self.inner.unlock();

                single_core::reenable_interrupts(token);
            }
        }
    }

    mod single_core {
        use core::sync::atomic::{compiler_fence, Ordering};

        pub unsafe fn disable_interrupts() -> critical_section::RawRestoreState {
            let token: critical_section::RawRestoreState;
            core::arch::asm!("rsil {0}, 5", out(reg) token);

            // Ensure no subsequent memory accesses are reordered to before interrupts are
            // disabled.
            compiler_fence(Ordering::SeqCst);

            token
        }

        pub unsafe fn reenable_interrupts(token: critical_section::RawRestoreState) {
            // Ensure no preceeding memory accesses are reordered to after interrupts are
            // enabled.
            compiler_fence(Ordering::SeqCst);

            // Reserved bits in the PS register, these must be written as 0.
            const RESERVED_MASK: u32 = 0b1111_1111_1111_1000_1111_0000_0000_0000;
            debug_assert!(token & RESERVED_MASK == 0);
            core::arch::asm!("wsr.ps {0}", "rsync", in(reg) token)
        }
    }

    mod multicore {
        use portable_atomic::{AtomicUsize, Ordering};

        // Safety: Ensure that when adding new chips `raw_core` doesn't return this
        // value.
        // FIXME: ensure in HIL tests this is the case!
        const UNUSED_THREAD_ID_VALUE: usize = 0x100;

        pub fn thread_id() -> usize {
            super::raw_core()
        }

        pub(super) struct AtomicLock {
            owner: AtomicUsize,
        }

        impl AtomicLock {
            pub const fn new() -> Self {
                Self {
                    owner: AtomicUsize::new(UNUSED_THREAD_ID_VALUE),
                }
            }

            pub fn is_owned_by_current_thread(&self) -> bool {
                self.is_owned_by(thread_id())
            }

            pub fn is_owned_by(&self, thread: usize) -> bool {
                self.owner.load(Ordering::Relaxed) == thread
            }

            pub fn try_lock(&self, new_owner: usize) -> Result<(), usize> {
                self.owner
                    .compare_exchange(
                        UNUSED_THREAD_ID_VALUE,
                        new_owner,
                        Ordering::Acquire,
                        Ordering::Relaxed,
                    )
                    .map(|_| ())
            }

            /// # Safety:
            ///
            /// This function must only be called if the lock was acquired by the
            /// current thread.
            pub unsafe fn unlock(&self) {
                debug_assert!(self.is_owned_by_current_thread());
                self.owner.store(UNUSED_THREAD_ID_VALUE, Ordering::Release);
            }
        }
    }

    fn raw_core() -> usize {
        (xtensa_lx::get_processor_id() & 0x2000) as usize
    }
}

/// Embassy-time implementation.
///
/// Stolen from the [esp-hal] crate.
///
/// [esp-hal]: https://github.com/esp-rs/esp-hal/blob/713cd491b6a6645bc8fe107d1e4d284135ca4459/esp-hal/src/time.rs
pub mod time {
    use esp32::TIMG0;

    /// Represents an instant in time.
    ///
    /// The resolution is 1 microsecond, represented as a 64-bit unsigned integer.
    pub type Instant = fugit::Instant<u64, 1, 1_000_000>;

    /// Provides time since system start in microseconds precision.
    ///
    /// The counter won’t measure time in sleep-mode.
    ///
    /// The timer will wrap after 36_558 years.
    pub fn now() -> Instant {
        let (ticks, div) = {
            // on ESP32 use LACT
            let tg0 = unsafe { TIMG0::steal() };
            tg0.lactupdate().write(|w| unsafe { w.update().bits(1) });

            // The peripheral doesn't have a bit to indicate that the update is done, so we
            // poll the lower 32 bit part of the counter until it changes, or a timeout
            // expires.
            let lo_initial = tg0.lactlo().read().bits();
            let mut div = tg0.lactconfig().read().divider().bits();
            let lo = loop {
                let lo = tg0.lactlo().read().bits();
                if lo != lo_initial || div == 0 {
                    break lo;
                }
                div -= 1;
            };
            let hi = tg0.lacthi().read().bits();

            let ticks = (hi as u64) << 32u64 | lo as u64;
            (ticks, 16)
        };

        Instant::from_ticks(ticks / div)
    }
}

mod embassy_time_driver_impl {
    use embassy_time_driver::Driver;

    struct EmbassyDriver;

    impl Driver for EmbassyDriver {
        fn now(&self) -> u64 {
            super::time::now().ticks()
        }

        unsafe fn allocate_alarm(&self) -> Option<embassy_time_driver::AlarmHandle> {
            unimplemented!()
        }

        fn set_alarm_callback(
            &self,
            _: embassy_time_driver::AlarmHandle,
            _: fn(*mut ()),
            _: *mut (),
        ) {
            unimplemented!()
        }

        fn set_alarm(&self, _: embassy_time_driver::AlarmHandle, _: u64) -> bool {
            unimplemented!()
        }
    }

    embassy_time_driver::time_driver_impl!(static DRIVER: EmbassyDriver = EmbassyDriver{});
}
