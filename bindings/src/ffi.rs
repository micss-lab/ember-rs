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

mod util {
    use alloc::string::{String, ToString};
    use core::ffi::{c_char, CStr};

    pub(super) fn new<T>(value: T) -> *mut T {
        use alloc::boxed::Box;
        Box::into_raw(Box::new(value))
    }

    pub(super) unsafe fn from_raw<T>(pointer: *mut T) -> T {
        use alloc::boxed::Box;
        *Box::from_raw(pointer)
    }

    pub(super) unsafe fn ref_from_raw<T>(pointer: *mut T) -> &'static mut T {
        &mut *pointer
    }

    pub(super) unsafe fn drop_raw<T>(pointer: *mut T) {
        use alloc::boxed::Box;
        drop(Box::from_raw(pointer));
    }

    pub(super) unsafe fn string_from_raw(string: *const c_char) -> String {
        let string = CStr::from_ptr(string);
        String::from_utf8_lossy(string.to_bytes()).to_string()
    }
}

mod container {
    use no_std_framework_core::{Agent, Container};

    use crate::ffi::util::drop_raw;

    use super::util::{from_raw, new};

    /// Creates a new container instance.
    ///
    /// # Safety
    ///
    /// The ownership of the instance is transferred to the caller. Make sure to free the memory
    /// with the accompanying [`container_free`].
    #[no_mangle]
    pub extern "C" fn container_new() -> *mut Container {
        log::trace!("Creating new container\r");
        new(Container::default())
    }

    // Wrapper to add an agent to the container
    #[no_mangle]
    pub extern "C" fn container_add_agent(container: *mut Container, agent: *mut Agent) {
        non_null!(container, "got container nullpointer");
        non_null!(agent, "got agent nullpointer");
        let agent = unsafe { from_raw(agent) };
        unsafe { (*container).add_agent(agent) };
    }

    // Wrapper to start the container
    #[no_mangle]
    pub extern "C" fn container_start(container: *mut Container) -> i32 {
        non_null!(container, "got container null-pointer");

        let result = unsafe { from_raw(container) }.start();
        match result {
            Ok(()) => 0,
            Err(_) => -1,
        }
    }

    // Wrapper to free the container
    #[no_mangle]
    pub extern "C" fn container_free(container: *mut Container) {
        non_null_or_bail!(container, "attemted to free container null-pointer");
        unsafe { drop_raw(container) }
    }
}

mod agent {
    use alloc::boxed::Box;
    use core::ffi::c_char;

    use no_std_framework_core::{behaviour::Behaviour, Agent};

    use super::util::{drop_raw, new, ref_from_raw, string_from_raw};

    #[no_mangle]
    pub extern "C" fn agent_new(name: *const c_char) -> *mut Agent {
        let name = unsafe { string_from_raw(name) };
        new(Agent::new(name))
    }

    // #[no_mangle]
    // pub extern "C" fn agent_add_behaviour(
    //     agent: *mut Agent,
    //     behaviour: *mut dyn Behaviour<ParentState = ()>,
    // ) {
    //     non_null!(agent, "got agent null-pointer");
    //     non_null!(agent, "got behaviour null-pointer");
    //     let agent = unsafe { ref_from_raw(agent) };
    //     let behaviour = unsafe { Box::from_raw(behaviour) };
    //     agent.add_boxed_behaviour(behaviour);
    // }

    #[no_mangle]
    pub extern "C" fn agent_free(agent: *mut Agent) {
        non_null_or_bail!(agent, "attemted to free agent null-pointer");
        unsafe { drop_raw(agent) }
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
