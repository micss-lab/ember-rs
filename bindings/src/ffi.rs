macro_rules! non_null_or_bail {
    ($value:expr, $message:literal) => {
        if $value.is_null() {
            log::warn!($message);
            return;
        }
    };
}

mod container {
    use alloc::boxed::Box;

    use no_std_framework_core::{Agent, Container};

    /// Creates a new container instance.
    ///
    /// # Safety
    ///
    /// The ownership of the instance is transferred to the caller. Make sure to free the memory
    /// with the accompanying [`container_free`].
    #[no_mangle]
    pub extern "C" fn container_new() -> *mut Container {
        log::info!("Hello from rust!\r");
        log::debug!("Creating new container\r");
        log::debug!("Doing some more work\r");
        let container = Container::default();
        Box::into_raw(Box::new(container))
    }

    // Wrapper to add an agent to the container
    #[no_mangle]
    pub unsafe extern "C" fn container_add_agent(container: *mut Container, agent: *mut Agent) {
        if !container.is_null() {
            unsafe {
                (*container).add_agent(*Box::from_raw(agent));
            }
        }
    }

    // Wrapper to start the container
    #[no_mangle]
    pub unsafe extern "C" fn container_start(container: *mut Container) -> i32 {
        if container.is_null() {
            return -1; // Error: Null pointer
        }

        let result = unsafe { Box::from_raw(container).start() };
        match result {
            Ok(_) => 0,   // Success
            Err(_) => -2, // Error
        }
    }

    // Wrapper to free the container
    #[no_mangle]
    pub unsafe extern "C" fn container_free(container: *mut Container) {
        non_null_or_bail!(container, "Attemted to free container null-pointer");
        drop(unsafe { Box::from_raw(container) });
    }
}

mod logging {
    use core::ffi::c_char;
    use log::LevelFilter;

    /// Initialize the libraries global logger.
    ///
    /// Values less or equal to 0 disable logging. Values from 1 to 5 (and up) set respectively the levels;
    /// error, warn, info, debug, trace.
    #[no_mangle]
    pub extern "C" fn initialize_logging(level: c_char) {
        crate::log::initialize_logging(match level.max(0) as u8 {
            0 => LevelFilter::Off,
            1 => LevelFilter::Error,
            2 => LevelFilter::Warn,
            3 => LevelFilter::Info,
            4 => LevelFilter::Debug,
            _ => LevelFilter::Trace,
        });
    }
}
