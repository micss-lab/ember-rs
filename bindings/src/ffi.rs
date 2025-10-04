macro_rules! non_null_or_bail {
    ($value:expr, $message:literal $(, $ret:expr)?) => {
        if $value.is_null() {
            log::warn!($message);
            return $($ret)?;
        }
    };
}

macro_rules! non_null {
    ($value:expr, $message:literal) => {
        if $value.is_null() {
            panic!($message);
        }
    };
}

/// cbindgen:ignore
mod util {
    use alloc::string::{String, ToString};
    use core::ffi::{CStr, c_char};

    pub(super) fn new<T>(value: T) -> *mut T {
        use alloc::boxed::Box;
        Box::into_raw(Box::new(value))
    }

    pub(super) unsafe fn from_raw<T>(pointer: *mut T) -> T {
        use alloc::boxed::Box;
        *unsafe { Box::from_raw(pointer) }
    }

    pub(super) unsafe fn ref_from_raw<'a, T>(pointer: *mut T) -> &'a mut T {
        unsafe { &mut *pointer }
    }

    pub(super) unsafe fn drop_raw<T>(pointer: *mut T) {
        use alloc::boxed::Box;
        drop(unsafe { Box::from_raw(pointer) });
    }

    pub(super) unsafe fn string_from_raw(string: *const c_char) -> String {
        let string = unsafe { CStr::from_ptr(string) };
        String::from_utf8_lossy(string.to_bytes()).to_string()
    }
}

#[cfg(target_os = "none")]
mod esp {
    #[unsafe(no_mangle)]
    pub extern "C" fn initialize_allocator() {
        crate::esp::initialize_allocator();
    }
}

mod event {
    use core::ffi::c_void;

    use super::util::{drop_raw, new};

    #[repr(C)]
    pub struct Event {
        inner: *mut c_void,
    }

    #[unsafe(no_mangle)]
    extern "C" fn event_new(event: *mut c_void) -> *mut Event {
        new(Event { inner: event })
    }

    #[unsafe(no_mangle)]
    extern "C" fn event_free(event: *mut Event) {
        non_null_or_bail!(event, "attempted to free event null-pointer");
        unsafe { drop_raw(event) }
    }
}

mod agent_state {
    use core::ffi::c_void;

    use super::util::{drop_raw, new};

    #[repr(C)]
    pub struct AgentState {
        inner: *mut c_void,
    }

    #[unsafe(no_mangle)]
    extern "C" fn agent_state_new(agent_state: *mut c_void) -> *mut AgentState {
        new(AgentState { inner: agent_state })
    }

    #[unsafe(no_mangle)]
    extern "C" fn agent_state_free(agent_state: *mut AgentState) {
        non_null_or_bail!(agent_state, "attempted to free agent state null-pointer");
        unsafe { drop_raw(agent_state) }
    }
}

mod agent;
mod behaviour;
mod container;
mod context;

mod logging {
    use core::ffi::c_char;
    use log::LevelFilter;

    /// Initialize the libraries global logger.
    ///
    /// Values less or equal to 0 disable logging. Values from 1 to 5 (and up) set respectively the levels;
    /// error, warn, info, debug, trace.
    #[unsafe(no_mangle)]
    pub extern "C" fn initialize_logging(level: c_char) {
        crate::log::initialize_logging(match level {
            0 => LevelFilter::Off,
            1 => LevelFilter::Error,
            2 => LevelFilter::Warn,
            3 => LevelFilter::Info,
            4 => LevelFilter::Debug,
            _ => LevelFilter::Trace,
        });
    }
}
