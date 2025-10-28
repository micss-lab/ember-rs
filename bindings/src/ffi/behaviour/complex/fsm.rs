use core::cell::Cell;
use core::ffi::{c_char, c_void};
use core::ptr;

use ember::behaviour::fsm::{Fsm, FsmBehaviour as FsmBehaviourTrait, FsmBuilder, FsmEvent};
use ember::behaviour::{BehaviourId, ComplexBehaviour, Context, IntoBehaviourWithId};

use crate::ffi::util::ref_from_raw;

use super::SequentialBehaviour;
use super::simple::{CyclicBehaviour, OneShotBehaviour, TickerBehaviour};
use super::{AgentState, Event};
use super::{drop_raw, from_raw, new};

pub struct FsmBehaviour<E> {
    /// Type value defined by the user implementing the trait.
    inner: *mut c_void,
    /// Inner finite-state machine controlling the execution of child behaviours.
    fsm: Cell<*mut Fsm<'static, AgentState, *const c_char, Event>>,
    /// Function to be executed for every event a child has emitted.
    handle_child_event: extern "C" fn(*mut c_void, *mut Event),
    /// Function to be executed after a child behaviour has performed its action.
    after_child_action: extern "C" fn(*mut c_void, *mut Context<E>, *mut AgentState),
}

impl<E> ComplexBehaviour for FsmBehaviour<E> {
    type AgentState = AgentState;

    type Event = E;

    type ChildEvent = Event;

    fn handle_child_event(&mut self, event: Self::ChildEvent) {
        (self.handle_child_event)(self.inner, new(event))
    }

    fn after_child_action(
        &mut self,
        ctx: &mut ember::behaviour::Context<Self::Event>,
        agent_state: &mut Self::AgentState,
    ) {
        (self.after_child_action)(self.inner, ptr::from_mut(ctx), ptr::from_mut(agent_state));
    }
}

impl<E: 'static> FsmBehaviourTrait<'static> for FsmBehaviour<E> {
    type TransitionTrigger = *const c_char;

    fn fsm(&self) -> Fsm<'static, Self::AgentState, Self::TransitionTrigger, Self::ChildEvent> {
        // Replace the fsm pointer with a null-pointer.
        non_null!(self.fsm.get(), "fsm can only be fetched once");
        let result = self.fsm.replace(ptr::null_mut());
        unsafe { from_raw(result) }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn behaviour_fsm_behaviour_new(
    inner: *mut c_void,
    fsm: *mut Fsm<'static, AgentState, *const c_char, Event>,
    handle_child_event: extern "C" fn(*mut c_void, *mut Event),
    after_child_action: extern "C" fn(*mut c_void, *mut Context<Event>, *mut AgentState),
) -> *mut FsmBehaviour<Event> {
    non_null!(inner, "got inner null-pointer");
    non_null!(fsm, "got fsm null-pointer");
    new(FsmBehaviour {
        inner,
        fsm: fsm.into(),
        handle_child_event,
        after_child_action,
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn behaviour_fsm_behaviour_free(fsm: *mut FsmBehaviour<Event>) {
    non_null_or_bail!(fsm, "attempted to free fsm behaviour null-pointer");
    unsafe { drop_raw(fsm) };
}

#[unsafe(no_mangle)]
pub extern "C" fn behaviour_fsm_builder_new()
-> *mut FsmBuilder<'static, AgentState, *const c_char, Event> {
    new(Fsm::builder())
}

#[unsafe(no_mangle)]
pub extern "C" fn behaviour_fsm_builder_add_behaviour_oneshot(
    builder: *mut FsmBuilder<'static, AgentState, *const c_char, Event>,
    oneshot: *mut OneShotBehaviour<FsmEvent<*const c_char, Event>>,
    is_final: bool,
) -> BehaviourId {
    non_null!(builder, "got fsm builder null-pointer");
    non_null!(oneshot, "got oneshot behaviour null-pointer");
    let oneshot = unsafe { from_raw(oneshot) };
    let (id, oneshot) = oneshot.into_behaviour_with_id();
    unsafe { ref_from_raw(builder) }.add_behaviour(oneshot, is_final);
    id
}

#[unsafe(no_mangle)]
pub extern "C" fn behaviour_fsm_builder_add_behaviour_cyclic(
    builder: *mut FsmBuilder<'static, AgentState, *const c_char, Event>,
    cyclic: *mut CyclicBehaviour<FsmEvent<*const c_char, Event>>,
    is_final: bool,
) -> BehaviourId {
    non_null!(builder, "got fsm builder null-pointer");
    non_null!(cyclic, "got cyclic behaviour null-pointer");
    let cyclic = unsafe { from_raw(cyclic) };
    let (id, cyclic) = cyclic.into_behaviour_with_id();
    unsafe { ref_from_raw(builder) }.add_behaviour(cyclic, is_final);
    id
}

#[unsafe(no_mangle)]
pub extern "C" fn behaviour_fsm_builder_add_behaviour_ticker(
    builder: *mut FsmBuilder<'static, AgentState, *const c_char, Event>,
    ticker: *mut TickerBehaviour<FsmEvent<*const c_char, Event>>,
    is_final: bool,
) -> BehaviourId {
    non_null!(builder, "got fsm builder null-pointer");
    non_null!(ticker, "got ticker behaviour null-pointer");
    let ticker = unsafe { from_raw(ticker) };
    let (id, ticker) = ticker.into_behaviour_with_id();
    unsafe { ref_from_raw(builder) }.add_behaviour(ticker, is_final);
    id
}

#[unsafe(no_mangle)]
pub extern "C" fn behaviour_fsm_builder_add_behaviour_sequential(
    builder: *mut FsmBuilder<'static, AgentState, *const c_char, Event>,
    sequential: *mut SequentialBehaviour<FsmEvent<*const c_char, Event>>,
    is_final: bool,
) -> BehaviourId {
    non_null!(builder, "got fsm builder null-pointer");
    non_null!(sequential, "got sequential behaviour null-pointer");
    let sequential = unsafe { from_raw(sequential) };
    let (id, sequential) = sequential.into_behaviour_with_id();
    unsafe { ref_from_raw(builder) }.add_behaviour(sequential, is_final);
    id
}

#[unsafe(no_mangle)]
pub extern "C" fn behaviour_fsm_builder_add_behaviour_fsm(
    builder: *mut FsmBuilder<'static, AgentState, *const c_char, Event>,
    fsm: *mut FsmBehaviour<FsmEvent<*const c_char, Event>>,
    is_final: bool,
) -> BehaviourId {
    non_null!(builder, "got fsm builder null-pointer");
    non_null!(fsm, "got fsm behaviour null-pointer");
    let fsm = unsafe { from_raw(fsm) };
    let (id, fsm) = fsm.into_behaviour_with_id();
    unsafe { ref_from_raw(builder) }.add_behaviour(fsm, is_final);
    id
}

#[unsafe(no_mangle)]
pub extern "C" fn behaviour_fsm_builder_build(
    builder: *mut FsmBuilder<'static, AgentState, *const c_char, Event>,
    start_behaviour: BehaviourId,
) -> *mut Fsm<'static, AgentState, *const c_char, Event> {
    non_null!(builder, "got fsm builder null-pointer");
    new(unsafe { from_raw(builder) }
        .try_build(start_behaviour)
        .expect("failed to build fsm"))
}

pub(in crate::ffi) mod fsm_child_behaviour {
    pub(in crate::ffi) mod simple {
        use core::ffi::{c_char, c_void};

        use ember::behaviour::{Context, fsm::FsmEvent};

        use crate::ffi::agent_state::AgentState;
        use crate::ffi::behaviour::simple::{CyclicBehaviour, OneShotBehaviour, TickerBehaviour};
        use crate::ffi::event::Event;
        use crate::ffi::util::{drop_raw, new};

        #[unsafe(no_mangle)]
        pub extern "C" fn behaviour_fsm_child_behaviour_oneshot_new(
            inner: *mut c_void,
            action: extern "C" fn(
                *mut c_void,
                *mut Context<FsmEvent<*const c_char, Event>>,
                *mut AgentState,
            ),
        ) -> *mut OneShotBehaviour<FsmEvent<*const c_char, Event>> {
            new(OneShotBehaviour { inner, action })
        }

        #[unsafe(no_mangle)]
        pub extern "C" fn behaviour_fsm_child_behaviour_oneshot_free(
            oneshot: *mut OneShotBehaviour<FsmEvent<*const c_char, Event>>,
        ) {
            non_null_or_bail!(oneshot, "attemted to free oneshot behaviour null-pointer");
            unsafe { drop_raw(oneshot) };
        }

        #[unsafe(no_mangle)]
        pub extern "C" fn behaviour_fsm_child_behaviour_cyclic_new(
            inner: *mut c_void,
            action: extern "C" fn(
                *mut c_void,
                *mut Context<FsmEvent<*const c_char, Event>>,
                *mut AgentState,
            ),
            is_finished: extern "C" fn(*mut c_void) -> bool,
        ) -> *mut CyclicBehaviour<FsmEvent<*const c_char, Event>> {
            new(CyclicBehaviour {
                inner,
                action,
                is_finished,
            })
        }

        #[unsafe(no_mangle)]
        pub extern "C" fn behaviour_fsm_child_behaviour_cyclic_free(
            cyclic: *mut CyclicBehaviour<FsmEvent<*const c_char, Event>>,
        ) {
            non_null_or_bail!(cyclic, "attempted to free cyclic behaviour null-pointer");
            unsafe { drop_raw(cyclic) };
        }

        #[unsafe(no_mangle)]
        pub extern "C" fn behaviour_fsm_child_behaviour_ticker_new(
            inner: *mut c_void,
            interval: extern "C" fn(*mut c_void) -> u64,
            action: extern "C" fn(
                *mut c_void,
                *mut Context<FsmEvent<*const c_char, Event>>,
                *mut AgentState,
            ),
            is_finished: extern "C" fn(*mut c_void) -> bool,
        ) -> *mut TickerBehaviour<FsmEvent<*const c_char, Event>> {
            new(TickerBehaviour {
                inner,
                action,
                is_finished,
                interval,
            })
        }

        #[unsafe(no_mangle)]
        pub extern "C" fn behaviour_fsm_child_behaviour_ticker_free(
            ticker: *mut TickerBehaviour<FsmEvent<*const c_char, Event>>,
        ) {
            non_null_or_bail!(ticker, "attempted to free ticker behaviour null-pointer");
            unsafe { drop_raw(ticker) };
        }
    }

    pub(in crate::ffi) mod complex {
        use core::ffi::{c_char, c_void};

        use ember::behaviour::Context;
        use ember::behaviour::fsm::{Fsm, FsmEvent};

        use crate::ffi::agent_state::AgentState;
        use crate::ffi::behaviour::complex::array::BehaviourVec;
        use crate::ffi::behaviour::complex::{FsmBehaviour, SequentialBehaviour};
        use crate::ffi::event::Event;
        use crate::ffi::util::{drop_raw, new};

        #[unsafe(no_mangle)]
        pub extern "C" fn behaviour_fsm_child_behaviour_sequential_new(
            inner: *mut c_void,
            initial_behaviours: *mut BehaviourVec<Event>,
            handle_child_event: extern "C" fn(*mut c_void, *mut Event),
            after_child_action: extern "C" fn(
                *mut c_void,
                *mut Context<FsmEvent<*const c_char, Event>>,
                *mut AgentState,
            ),
        ) -> *mut SequentialBehaviour<FsmEvent<*const c_char, Event>> {
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
        pub extern "C" fn behaviour_fsm_child_behaviour_sequential_free(
            sequential: *mut SequentialBehaviour<FsmEvent<*const c_char, Event>>,
        ) {
            non_null_or_bail!(
                sequential,
                "attempted to free sequential behaviour null-pointer"
            );
            unsafe { drop_raw(sequential) };
        }

        #[unsafe(no_mangle)]
        pub extern "C" fn behaviour_fsm_child_behaviour_fsm_new(
            inner: *mut c_void,
            fsm: *mut Fsm<'static, AgentState, *const c_char, Event>,
            handle_child_event: extern "C" fn(*mut c_void, *mut Event),
            after_child_action: extern "C" fn(
                *mut c_void,
                *mut Context<FsmEvent<*const c_char, Event>>,
                *mut AgentState,
            ),
        ) -> *mut FsmBehaviour<FsmEvent<*const c_char, Event>> {
            non_null!(inner, "got inner null-pointer");
            non_null!(fsm, "got fsm null-pointer");
            new(FsmBehaviour {
                inner,
                fsm: fsm.into(),
                handle_child_event,
                after_child_action,
            })
        }

        #[unsafe(no_mangle)]
        pub extern "C" fn behaviour_fsm_child_behaviour_fsm_free(
            fsm: *mut FsmBehaviour<FsmEvent<*const c_char, Event>>,
        ) {
            non_null_or_bail!(fsm, "attempted to free fsm behaviour null-pointer");
            unsafe { drop_raw(fsm) };
        }
    }
}
