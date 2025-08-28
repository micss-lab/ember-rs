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

mod container {
    use ember::{Agent, Container};

    use crate::ffi::util::{drop_raw, ref_from_raw};

    use super::agent_state::AgentState;
    use super::event::Event;
    use super::util::{from_raw, new};

    /// Creates a new container instance.
    ///
    /// # Safety
    ///
    /// The ownership of the instance is transferred to the caller. Make sure to free the memory
    /// with the accompanying [`container_free`].
    #[unsafe(no_mangle)]
    pub extern "C" fn container_new() -> *mut Container<'static, 'static> {
        log::trace!("Creating new container");
        new(Container::default())
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn container_free(container: *mut Container) {
        non_null_or_bail!(container, "attemted to free container null-pointer");
        unsafe { drop_raw(container) }
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn container_add_agent(
        container: *mut Container,
        agent: *mut Agent<'static, AgentState, Event>,
    ) {
        non_null!(container, "got container null-pointer");
        non_null!(agent, "got agent null-pointer");
        let agent = unsafe { from_raw(agent) };
        unsafe { (*container).add_agent(agent) };
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn container_start(container: *mut Container) -> i32 {
        non_null!(container, "got container null-pointer");
        let result = unsafe { from_raw(container) }.start();
        match result {
            Ok(()) => 0,
            Err(_) => 1,
        }
    }

    #[repr(C)]
    pub struct ContainerPollResult {
        status: i32,
        should_stop: bool,
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn container_poll(container: *mut Container) -> ContainerPollResult {
        non_null!(container, "got container null-pointer");
        let container = unsafe { ref_from_raw(container) };
        let (should_stop, status) = match container.poll() {
            Ok(should_stop) => (should_stop, 0),
            Err(_) => (true, 1),
        };
        ContainerPollResult {
            status,
            should_stop,
        }
    }
}

mod agent {
    use core::ffi::c_char;

    use ember::Agent;

    use super::agent_state::AgentState;
    use super::behaviour::complex::SequentialBehaviour;
    use super::behaviour::simple::{CyclicBehaviour, OneShotBehaviour, TickerBehaviour};
    use super::event::Event;
    use super::util::{drop_raw, from_raw, new, ref_from_raw, string_from_raw};

    #[unsafe(no_mangle)]
    pub extern "C" fn agent_new(
        name: *const c_char,
        agent_state: *mut AgentState,
    ) -> *mut Agent<'static, AgentState, Event> {
        let name = unsafe { string_from_raw(name) };
        non_null!(agent_state, "got agent state null-pointer");
        let agent_state = unsafe { from_raw(agent_state) };
        new(Agent::new(name, agent_state))
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn agent_free(agent: *mut Agent<AgentState, Event>) {
        non_null_or_bail!(agent, "attemted to free agent null-pointer");
        unsafe { drop_raw(agent) }
    }

    // TODO: Add more behaviours here.
    #[unsafe(no_mangle)]
    pub extern "C" fn agent_add_behaviour_oneshot(
        agent: *mut Agent<AgentState, Event>,
        oneshot: *mut OneShotBehaviour,
    ) {
        non_null!(agent, "got agent null-pointer");
        non_null!(oneshot, "got oneshot behaviour null-pointer");
        let agent = unsafe { ref_from_raw(agent) };
        let behaviour = unsafe { from_raw(oneshot) };
        agent.add_behaviour(behaviour);
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn agent_add_behaviour_cyclic(
        agent: *mut Agent<AgentState, Event>,
        cyclic: *mut CyclicBehaviour,
    ) {
        non_null!(agent, "got agent null-pointer");
        non_null!(cyclic, "got cyclic behaviour null-pointer");
        let agent = unsafe { ref_from_raw(agent) };
        let behaviour = unsafe { from_raw(cyclic) };
        agent.add_behaviour(behaviour);
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn agent_add_behaviour_ticker(
        agent: *mut Agent<AgentState, Event>,
        ticker: *mut TickerBehaviour,
    ) {
        non_null!(agent, "got agent null-pointer");
        non_null!(ticker, "got ticker behaviour null-pointer");
        let agent = unsafe { ref_from_raw(agent) };
        let behaviour = unsafe { from_raw(ticker) };
        agent.add_behaviour(behaviour);
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn agent_add_behaviour_sequential(
        agent: *mut Agent<'static, AgentState, Event>,
        sequential: *mut SequentialBehaviour,
    ) {
        non_null!(agent, "got agent null-pointer");
        non_null!(sequential, "got sequential behaviour null-pointer");
        let agent = unsafe { ref_from_raw(agent) };
        let behaviour = unsafe { from_raw(sequential) };
        agent.add_behaviour(behaviour);
    }
}

mod context {
    use ember::behaviour::Context;

    use super::event::Event;
    use super::util::{from_raw, ref_from_raw};

    // No `new` or `free` needed as this is a mutable borrow from rust.

    #[unsafe(no_mangle)]
    pub extern "C" fn context_emit_event(context: *mut Context<Event>, event: *mut Event) {
        non_null!(context, "got a context null-pointer");
        non_null!(event, "got a event null-pointer");
        let context = unsafe { ref_from_raw(context) };
        let event = unsafe { from_raw(event) };
        context.emit_event(event);
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn context_stop_container(context: *mut Context<Event>) {
        non_null!(context, "got a context null-pointer");
        let context = unsafe { ref_from_raw(context) };
        context.stop_container();
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn context_remove_agent(context: *mut Context<Event>) {
        non_null!(context, "got a context null-pointer");
        let context = unsafe { ref_from_raw(context) };
        context.remove_agent();
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn context_block_behaviour(context: *mut Context<Event>) {
        non_null!(context, "got a context null-pointer");
        let context = unsafe { ref_from_raw(context) };
        context.block_behaviour();
    }

    #[allow(unused)] // Rust thinks the variants are unused.
    #[repr(C)]
    pub enum ScheduleStrategy {
        Next,
        End,
    }
}

mod behaviour {
    use super::agent_state::AgentState;
    use super::event::Event;

    pub(super) mod simple {
        pub(in crate::ffi) use self::cyclic::CyclicBehaviour;
        pub(in crate::ffi) use self::oneshot::OneShotBehaviour;
        pub(in crate::ffi) use self::ticker::TickerBehaviour;

        use super::{AgentState, Event};
        use crate::ffi::util::{drop_raw, new};

        mod oneshot {
            use core::ffi::c_void;
            use core::ptr;

            use ember::behaviour::{Context, OneShotBehaviour as OneShotBehaviourTrait};

            use super::{AgentState, Event};
            use super::{drop_raw, new};

            pub struct OneShotBehaviour {
                /// Type value defined by the user implementing the trait.
                inner: *mut c_void,
                /// Action to be performed.
                action: extern "C" fn(*mut c_void, *mut Context<Event>, *mut AgentState),
            }

            impl OneShotBehaviourTrait for OneShotBehaviour {
                type AgentState = AgentState;

                type Event = Event;

                fn action(
                    &self,
                    ctx: &mut Context<Self::Event>,
                    agent_state: &mut Self::AgentState,
                ) {
                    (self.action)(self.inner, ptr::from_mut(ctx), ptr::from_mut(agent_state))
                }
            }

            #[unsafe(no_mangle)]
            pub extern "C" fn behaviour_oneshot_new(
                inner: *mut c_void,
                action: extern "C" fn(*mut c_void, *mut Context<Event>, *mut AgentState),
            ) -> *mut OneShotBehaviour {
                new(OneShotBehaviour { inner, action })
            }

            #[unsafe(no_mangle)]
            pub extern "C" fn behaviour_oneshot_free(oneshot: *mut OneShotBehaviour) {
                non_null_or_bail!(oneshot, "attemted to free oneshot behaviour null-pointer");
                unsafe { drop_raw(oneshot) };
            }
        }

        mod cyclic {
            use core::ffi::c_void;
            use core::ptr;

            use ember::behaviour::{Context, CyclicBehaviour as CyclicBehaviourTrait};

            use super::{AgentState, Event};
            use super::{drop_raw, new};

            pub struct CyclicBehaviour {
                /// Type value defined by the user implementing the trait.
                inner: *mut c_void,
                /// Action to be performed.
                action: extern "C" fn(*mut c_void, *mut Context<Event>, *mut AgentState),
                /// Whether the behaviour has finished.
                is_finished: extern "C" fn(*mut c_void) -> bool,
            }

            impl CyclicBehaviourTrait for CyclicBehaviour {
                type AgentState = AgentState;

                type Event = Event;

                fn action(
                    &mut self,
                    ctx: &mut Context<Self::Event>,
                    agent_state: &mut Self::AgentState,
                ) {
                    (self.action)(self.inner, ptr::from_mut(ctx), ptr::from_mut(agent_state));
                }

                fn is_finished(&self) -> bool {
                    (self.is_finished)(self.inner)
                }
            }

            #[unsafe(no_mangle)]
            pub extern "C" fn behaviour_cyclic_new(
                inner: *mut c_void,
                action: extern "C" fn(*mut c_void, *mut Context<Event>, *mut AgentState),
                is_finished: extern "C" fn(*mut c_void) -> bool,
            ) -> *mut CyclicBehaviour {
                new(CyclicBehaviour {
                    inner,
                    action,
                    is_finished,
                })
            }

            #[unsafe(no_mangle)]
            pub extern "C" fn behaviour_cyclic_free(cyclic: *mut CyclicBehaviour) {
                non_null_or_bail!(cyclic, "attemted to free cyclic behaviour null-pointer");
                unsafe { drop_raw(cyclic) };
            }
        }

        mod ticker {
            use core::ffi::c_void;
            use core::ptr;
            use core::time::Duration;

            use ember::behaviour::{Context, TickerBehaviour as TickerBehaviourTrait};

            use super::{AgentState, Event};
            use super::{drop_raw, new};

            pub struct TickerBehaviour {
                /// Type value defined by the user implementing the trait.
                inner: *mut c_void,
                /// Action to be performed.
                action: extern "C" fn(*mut c_void, *mut Context<Event>, *mut AgentState),
                /// Whether the behaviour has finished.
                is_finished: extern "C" fn(*mut c_void) -> bool,
                /// Interval in miliseconds until the next scheduled action.
                interval: extern "C" fn(*mut c_void) -> u64,
            }

            impl TickerBehaviourTrait for TickerBehaviour {
                type AgentState = AgentState;

                type Event = Event;

                fn interval(&self) -> Duration {
                    Duration::from_millis((self.interval)(self.inner))
                }

                fn action(
                    &mut self,
                    ctx: &mut Context<Self::Event>,
                    agent_state: &mut Self::AgentState,
                ) {
                    (self.action)(self.inner, ptr::from_mut(ctx), ptr::from_mut(agent_state));
                }

                fn is_finished(&self) -> bool {
                    (self.is_finished)(self.inner)
                }
            }

            #[unsafe(no_mangle)]
            pub extern "C" fn behaviour_ticker_new(
                inner: *mut c_void,
                interval: extern "C" fn(*mut c_void) -> u64,
                action: extern "C" fn(*mut c_void, *mut Context<Event>, *mut AgentState),
                is_finished: extern "C" fn(*mut c_void) -> bool,
            ) -> *mut TickerBehaviour {
                new(TickerBehaviour {
                    inner,
                    interval,
                    action,
                    is_finished,
                })
            }

            #[unsafe(no_mangle)]
            pub extern "C" fn behaviour_ticker_free(ticker: *mut TickerBehaviour) {
                non_null_or_bail!(ticker, "attemted to free ticker behaviour null-pointer");
                unsafe { drop_raw(ticker) };
            }
        }
    }

    pub(super) mod complex {
        pub(in crate::ffi) use self::sequential::SequentialBehaviour;

        use super::simple;
        use super::{AgentState, Event};
        use crate::ffi::util::{drop_raw, from_raw, new, ref_from_raw};

        mod array {
            use alloc::boxed::Box;
            use alloc::vec::Vec;

            use ember::behaviour::{Behaviour, IntoBehaviour};

            use super::sequential::SequentialBehaviour;
            use super::simple::{CyclicBehaviour, OneShotBehaviour, TickerBehaviour};
            use super::{AgentState, Event};
            use super::{drop_raw, from_raw, new, ref_from_raw};

            pub struct BehaviourVec(
                Vec<Box<dyn Behaviour<AgentState = AgentState, Event = Event>>>,
            );

            impl BehaviourVec {
                fn new() -> Self {
                    BehaviourVec(Vec::new())
                }

                fn add_behaviour<K>(
                    &mut self,
                    behaviour: impl IntoBehaviour<'static, K, AgentState = AgentState, Event = Event>,
                ) {
                    self.0.push(behaviour.into_behaviour());
                }
            }

            impl IntoIterator for BehaviourVec {
                type Item = Box<dyn Behaviour<AgentState = AgentState, Event = Event>>;

                type IntoIter = alloc::vec::IntoIter<Self::Item>;

                fn into_iter(self) -> Self::IntoIter {
                    self.0.into_iter()
                }
            }

            #[unsafe(no_mangle)]
            pub extern "C" fn behaviour_vec_new() -> *mut BehaviourVec {
                new(BehaviourVec::new())
            }

            #[unsafe(no_mangle)]
            pub extern "C" fn behaviour_vec_add_behaviour_oneshot(
                behaviour_vec: *mut BehaviourVec,
                oneshot: *mut OneShotBehaviour,
            ) {
                non_null!(behaviour_vec, "got sequential behaviour vec null-pointer");
                non_null!(oneshot, "got oneshot behaviour null-pointer");
                let behaviour_vec = unsafe { ref_from_raw(behaviour_vec) };
                let behaviour = unsafe { from_raw(oneshot) };
                behaviour_vec.add_behaviour(behaviour);
            }

            #[unsafe(no_mangle)]
            pub extern "C" fn behaviour_vec_add_behaviour_cyclic(
                behaviour_vec: *mut BehaviourVec,
                cyclic: *mut CyclicBehaviour,
            ) {
                non_null!(behaviour_vec, "got sequential behaviour vec null-pointer");
                non_null!(cyclic, "got cyclic behaviour null-pointer");
                let behaviour_vec = unsafe { ref_from_raw(behaviour_vec) };
                let behaviour = unsafe { from_raw(cyclic) };
                behaviour_vec.add_behaviour(behaviour);
            }

            #[unsafe(no_mangle)]
            pub extern "C" fn behaviour_vec_add_behaviour_ticker(
                behaviour_vec: *mut BehaviourVec,
                ticker: *mut TickerBehaviour,
            ) {
                non_null!(behaviour_vec, "got sequential behaviour vec null-pointer");
                non_null!(ticker, "got ticker behaviour null-pointer");
                let behaviour_vec = unsafe { ref_from_raw(behaviour_vec) };
                let behaviour = unsafe { from_raw(ticker) };
                behaviour_vec.add_behaviour(behaviour);
            }

            #[unsafe(no_mangle)]
            pub extern "C" fn behaviour_vec_add_behaviour_sequential(
                behaviour_vec: *mut BehaviourVec,
                sequential: *mut SequentialBehaviour,
            ) {
                non_null!(behaviour_vec, "got sequential behaviour vec null-pointer");
                non_null!(sequential, "got sequential behaviour null-pointer");
                let behaviour_vec = unsafe { ref_from_raw(behaviour_vec) };
                let behaviour = unsafe { from_raw(sequential) };
                behaviour_vec.add_behaviour(behaviour);
            }

            #[unsafe(no_mangle)]
            pub extern "C" fn behaviour_vec_free(behaviour_vec: *mut BehaviourVec) {
                non_null_or_bail!(
                    behaviour_vec,
                    "attemted to free sequential behaviour behaviour vec null-pointer"
                );
                unsafe { drop_raw(behaviour_vec) };
            }
        }

        mod sequential {
            use alloc::boxed::Box;
            use core::cell::Cell;
            use core::ffi::c_void;
            use core::ptr;

            use ember::behaviour::{
                Behaviour, ComplexBehaviour, Context,
                sequential::SequentialBehaviour as SequentialBehaviourTrait,
            };

            use super::array::BehaviourVec;
            use super::{AgentState, Event};
            use super::{drop_raw, from_raw, new};

            pub struct SequentialBehaviour {
                /// Type value defined by the user implementing the trait.
                inner: *mut c_void,
                /// List of initial behaviours to be scheduled.
                initial_behaviours: Cell<*mut BehaviourVec>,
                /// Function to be executed for every event a child has emitted.
                handle_child_event: extern "C" fn(*mut c_void, *mut Event),
                /// Function to be executed after a child behaviour has performed its action.
                after_child_action:
                    extern "C" fn(*mut c_void, *mut Context<Event>, *mut AgentState),
            }

            impl ComplexBehaviour for SequentialBehaviour {
                type AgentState = AgentState;

                type Event = Event;

                type ChildEvent = Event;

                fn handle_child_event(&mut self, event: Self::ChildEvent) {
                    (self.handle_child_event)(self.inner, new(event))
                }

                fn after_child_action(
                    &mut self,
                    ctx: &mut ember::behaviour::Context<Self::Event>,
                    agent_state: &mut Self::AgentState,
                ) {
                    (self.after_child_action)(
                        self.inner,
                        ptr::from_mut(ctx),
                        ptr::from_mut(agent_state),
                    );
                }
            }

            impl SequentialBehaviourTrait<'static> for SequentialBehaviour {
                fn initial_behaviours(
                    &self,
                ) -> impl IntoIterator<
                    Item = Box<
                        dyn Behaviour<AgentState = Self::AgentState, Event = Self::ChildEvent>,
                    >,
                > {
                    // Replace the initial behaviours pointer with a null-pointer.
                    non_null!(
                        self.initial_behaviours.get(),
                        "initial behaviours can only be fetched once"
                    );
                    let result = self.initial_behaviours.replace(ptr::null_mut());
                    unsafe { from_raw(result) }
                }
            }

            #[unsafe(no_mangle)]
            pub extern "C" fn behaviour_sequential_new(
                inner: *mut c_void,
                initial_behaviours: *mut BehaviourVec,
                handle_child_event: extern "C" fn(*mut c_void, *mut Event),
                after_child_action: extern "C" fn(
                    *mut c_void,
                    *mut Context<Event>,
                    *mut AgentState,
                ),
            ) -> *mut SequentialBehaviour {
                non_null!(inner, "got inner null-pointer");
                non_null!(initial_behaviours, "got initial behaviours null-pointer");
                new(SequentialBehaviour {
                    inner,
                    initial_behaviours: initial_behaviours.into(),
                    handle_child_event,
                    after_child_action,
                })
            }

            #[unsafe(no_mangle)]
            pub extern "C" fn behaviour_sequential_free(sequential: *mut SequentialBehaviour) {
                non_null_or_bail!(
                    sequential,
                    "attempted to free sequential behaviour null-pointer"
                );
                unsafe { drop_raw(sequential) };
            }
        }
    }
}

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
