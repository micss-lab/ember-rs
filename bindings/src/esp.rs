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

/// Embassy-time implementation.
///
/// Stolen from the [esp-hal] crate.
///
/// [esp-hal]: https://github.com/esp-rs/esp-hal/blob/713cd491b6a6645bc8fe107d1e4d284135ca4459/esp-hal/src/time.rs
pub mod time {
    /// Represents an instant in time.
    ///
    /// The resolution is 1 microsecond, represented as a 64-bit unsigned integer.
    pub type Instant = fugit::Instant<u64, 1, 1_000_000>;

    mod clock {
        use fugit::HertzU32;

        pub fn xtal_freq() -> HertzU32 {
            // Heavily reduced...

            HertzU32::MHz(40)
        }
    }

    mod systimer {
        use esp32c3::SYSTIMER;

        /// System Timer driver.
        pub struct SystemTimer;

        impl SystemTimer {
            /// Returns the tick frequency of the underlying timer unit.
            pub fn ticks_per_second() -> u64 {
                // The counters and comparators are driven using `XTAL_CLK`.
                // The average clock frequency is fXTAL_CLK/2.5, which is 16 MHz.
                // The timer counting is incremented by 1/16 μs on each `CNT_CLK` cycle.
                const MULTIPLIER: u64 = 10_000_000 / 25;

                let xtal_freq_mhz = super::clock::xtal_freq().to_MHz();
                xtal_freq_mhz as u64 * MULTIPLIER
            }

            /// Get the current count of the given unit in the System Timer.
            pub fn unit_value(unit: Unit) -> u64 {
                // This should be safe to access from multiple contexts
                // worst case scenario the second accessor ends up reading
                // an older time stamp

                unit.read_count()
            }
        }

        /// A 52-bit counter.
        #[derive(Copy, Clone, Debug, PartialEq, Eq)]
        pub enum Unit {
            /// Unit 0
            Unit0 = 0,
        }

        impl Unit {
            #[inline]
            fn channel(&self) -> u8 {
                *self as _
            }

            pub fn read_count(&self) -> u64 {
                // This can be a shared reference as long as this type isn't Sync.

                let channel = self.channel() as usize;
                let systimer = unsafe { SYSTIMER::steal() };

                systimer.unit_op(channel).write(|w| w.update().set_bit());
                while !systimer.unit_op(channel).read().value_valid().bit_is_set() {}

                // Read LO, HI, then LO again, check that LO returns the same value.
                // This accounts for the case when an interrupt may happen between reading
                // HI and LO values (or the other core updates the counter mid-read), and this
                // function may get called from the ISR. In this case, the repeated read
                // will return consistent values.
                let unit_value = systimer.unit_value(channel);
                let mut lo_prev = unit_value.lo().read().bits();
                loop {
                    let lo = lo_prev;
                    let hi = unit_value.hi().read().bits();
                    lo_prev = unit_value.lo().read().bits();

                    if lo == lo_prev {
                        return ((hi as u64) << 32) | lo as u64;
                    }
                }
            }
        }
    }

    /// Provides time since system start in microseconds precision.
    ///
    /// The counter won’t measure time in sleep-mode.
    ///
    /// The timer will wrap after 36_558 years.
    pub fn now() -> Instant {
        let (ticks, div) = {
            use self::systimer::{SystemTimer, Unit};
            // otherwise use SYSTIMER
            let ticks = SystemTimer::unit_value(Unit::Unit0);
            (ticks, (SystemTimer::ticks_per_second() / 1_000_000))
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
