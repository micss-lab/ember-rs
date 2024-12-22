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

#[cfg(target_os = "none")]
mod esp {
    #[no_mangle]
    pub extern "C" fn initialize_allocator() {
        crate::esp::initialize_allocator();
    }
}

mod message {
    use core::ffi::c_void;

    pub struct Message {
        inner: *mut c_void,
    }
}

mod container {
    use no_std_framework_core::{Agent, Container};

    use crate::ffi::util::drop_raw;

    use super::message::Message;
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

    #[no_mangle]
    pub extern "C" fn container_free(container: *mut Container) {
        non_null_or_bail!(container, "attemted to free container null-pointer");
        unsafe { drop_raw(container) }
    }

    #[no_mangle]
    pub extern "C" fn container_add_agent(container: *mut Container, agent: *mut Agent<Message>) {
        non_null!(container, "got container nullpointer");
        non_null!(agent, "got agent nullpointer");
        let agent = unsafe { from_raw(agent) };
        unsafe { (*container).add_agent(agent) };
    }

    #[no_mangle]
    pub extern "C" fn container_start(container: *mut Container) -> i32 {
        non_null!(container, "got container null-pointer");
        let result = unsafe { from_raw(container) }.start();
        match result {
            Ok(()) => 0,
            Err(_) => 1,
        }
    }
}

mod agent {
    use core::ffi::c_char;

    use no_std_framework_core::Agent;

    use super::behaviour::simple::OneShotBehaviour;
    use super::message::Message;
    use super::util::{drop_raw, from_raw, new, ref_from_raw, string_from_raw};

    #[no_mangle]
    pub extern "C" fn agent_new(name: *const c_char) -> *mut Agent<Message> {
        let name = unsafe { string_from_raw(name) };
        new(Agent::new(name))
    }

    #[no_mangle]
    pub extern "C" fn agent_free(agent: *mut Agent<Message>) {
        non_null_or_bail!(agent, "attemted to free agent null-pointer");
        unsafe { drop_raw(agent) }
    }

    // TODO: Add more behaviours here.
    #[no_mangle]
    pub extern "C" fn agent_add_behaviour_oneshot(
        agent: *mut Agent<Message>,
        oneshot: *mut OneShotBehaviour,
    ) {
        non_null!(agent, "got agent null-pointer");
        let agent = unsafe { ref_from_raw(agent) };
        let behaviour = unsafe { from_raw(oneshot) };
        agent.add_behaviour(behaviour)
    }
    //
    // #[no_mangle]
    // pub extern "C" fn agent_add_behaviour_cyclic(
    //     agent: *mut Agent,
    //     cyclic: *mut CyclicBehaviour<SimpleState, ()>,
    // ) {
    //     non_null!(agent, "got agent null-pointer");
    //     let agent = unsafe { ref_from_raw(agent) };
    //     let behaviour = unsafe { Box::from_raw(cyclic) };
    //     agent.add_boxed_behaviour(behaviour as Box<dyn Behaviour<ParentState = ()>>)
    // }
}

mod behaviour {
    use super::message::Message;

    pub(super) mod simple {
        pub(in crate::ffi) use oneshot::OneShotBehaviour;

        use super::Message;
        use crate::ffi::util::{drop_raw, new};

        mod oneshot {
            use core::ffi::c_void;
            use core::ptr;

            use no_std_framework_core::behaviour::{
                Context, OneShotBehaviour as OneShotBehaviourTrait,
            };

            use super::{drop_raw, new, Message};

            pub struct OneShotBehaviour {
                inner: *mut c_void,
                /// Action to be performed.
                ///
                /// First argument is the `inner` void pointer.
                action: extern "C" fn(*mut c_void, *mut Context<Message>),
            }

            impl OneShotBehaviourTrait for OneShotBehaviour {
                type Message = Message;

                fn action(&self, ctx: &mut Context<Self::Message>) {
                    (self.action)(self.inner, ptr::from_mut(ctx))
                }
            }

            #[no_mangle]
            pub extern "C" fn behaviour_oneshot_new(
                inner: *mut c_void,
                action: extern "C" fn(*mut c_void, *mut Context<Message>),
            ) -> *mut OneShotBehaviour {
                new(OneShotBehaviour { inner, action })
            }

            #[no_mangle]
            pub extern "C" fn behaviour_oneshot_free(oneshot: *mut OneShotBehaviour) {
                non_null_or_bail!(oneshot, "attemted to free oneshot behaviour null-pointer");
                unsafe { drop_raw(oneshot) };
            }
        }

        //     mod cyclic {
        //         use core::ptr;
        //
        //         use no_std_framework_core::behaviour::{Context, CyclicBehaviour};
        //
        //         use super::{drop_raw, new, SimpleState, State};
        //
        //         #[no_mangle]
        //         pub extern "C" fn behaviour_cyclic_new(
        //             state: SimpleState,
        //             action: extern "C" fn(*mut Context, *mut SimpleState, State) -> State,
        //         ) -> *mut CyclicBehaviour<SimpleState, State> {
        //             new(CyclicBehaviour::new(state, move |ctx, state| {
        //                 let (mut state, parent) = state.cut_root();
        //                 let parent = (action)(ptr::from_mut(ctx), ptr::from_mut(&mut state), parent);
        //                 no_std_framework_core::behaviour::State::new(state, parent)
        //             }))
        //         }
        //
        //         #[no_mangle]
        //         pub extern "C" fn behaviour_cyclic_new_void(
        //             state: SimpleState,
        //             action: extern "C" fn(*mut Context, *mut SimpleState),
        //         ) -> *mut CyclicBehaviour<SimpleState, ()> {
        //             new(CyclicBehaviour::new(state, move |ctx, state| {
        //                 let (mut state, _) = state.cut_root();
        //                 (action)(ptr::from_mut(ctx), ptr::from_mut(&mut state));
        //                 no_std_framework_core::behaviour::State::new(state, ())
        //             }))
        //         }
        //
        //         #[no_mangle]
        //         pub extern "C" fn behaviour_cyclic_free(
        //             cyclic: *mut CyclicBehaviour<SimpleState, State>,
        //         ) {
        //             non_null_or_bail!(cyclic, "attemted to free cyclic behaviour null-pointer");
        //             unsafe { drop_raw(cyclic) };
        //         }
        //
        //         #[no_mangle]
        //         pub extern "C" fn behaviour_cyclic_free_void(
        //             cyclic: *mut CyclicBehaviour<SimpleState, ()>,
        //         ) {
        //             non_null_or_bail!(cyclic, "attemted to free cyclic behaviour null-pointer");
        //             unsafe { drop_raw(cyclic) };
        //         }
        //     }
        // }
        //
        // mod complex {
        //     use super::super::util;
        //     use super::State;
        //
        //     mod sequential {
        //         use core::ffi::c_void;
        //
        //         use no_std_framework_core::behaviour::SequentialBehaviour;
        //
        //         use super::util::{drop_raw, new};
        //         use super::State;
        //
        //         #[no_mangle]
        //         pub extern "C" fn behaviour_sequential_new(
        //             state: *mut c_void,
        //         ) -> *mut SequentialBehaviour<*mut c_void, State> {
        //             new(SequentialBehaviour::new(state))
        //         }
        //
        //         #[no_mangle]
        //         pub extern "C" fn behaviour_sequential_new_void(
        //             state: *mut c_void,
        //         ) -> *mut SequentialBehaviour<*mut c_void, ()> {
        //             new(SequentialBehaviour::new(state))
        //         }
        //
        //         #[no_mangle]
        //         pub extern "C" fn behaviour_sequential_free(
        //             sequential: *mut SequentialBehaviour<*mut c_void, State>,
        //         ) {
        //             non_null_or_bail!(
        //                 sequential,
        //                 "attempted to free sequential behaviour null-pointer"
        //             );
        //             unsafe { drop_raw(sequential) };
        //         }
        //
        //         #[no_mangle]
        //         pub extern "C" fn behaviour_sequential_free_void(
        //             sequential: *mut SequentialBehaviour<*mut c_void, ()>,
        //         ) {
        //             non_null_or_bail!(
        //                 sequential,
        //                 "attempted to free sequential behaviour null-pointer"
        //             );
        //             unsafe { drop_raw(sequential) };
        //         }
        //     }
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
