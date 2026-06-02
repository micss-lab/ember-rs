use core::cell::Cell;

#[repr(transparent)]
pub struct AtomicU32(Cell<u32>);

// SAFETY: Internal methods are protected using the [`critical-section`] crate.
unsafe impl Sync for AtomicU32 {}

impl AtomicU32 {
    pub const fn new(value: u32) -> Self {
        Self(Cell::new(value))
    }

    pub fn get_increment(&self) -> u32 {
        critical_section::with(|_| {
            let value = self.0.get();
            self.0
                .replace(value.checked_add(1).expect("atomic u32 overflow"))
        })
    }
}
