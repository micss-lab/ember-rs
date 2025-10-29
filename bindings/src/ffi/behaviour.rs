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

        pub struct OneShotBehaviour<E> {
            /// Type value defined by the user implementing the trait.
            pub(in crate::ffi::behaviour) inner: *mut c_void,
            /// Action to be performed.
            pub(in crate::ffi::behaviour) action:
                extern "C" fn(*mut c_void, *mut Context<E>, *mut AgentState),
        }

        impl<E> OneShotBehaviourTrait for OneShotBehaviour<E> {
            type AgentState = AgentState;

            type Event = E;

            fn action(&self, ctx: &mut Context<Self::Event>, agent_state: &mut Self::AgentState) {
                (self.action)(self.inner, ptr::from_mut(ctx), ptr::from_mut(agent_state))
            }
        }

        #[unsafe(no_mangle)]
        pub extern "C" fn behaviour_oneshot_new(
            inner: *mut c_void,
            action: extern "C" fn(*mut c_void, *mut Context<Event>, *mut AgentState),
        ) -> *mut OneShotBehaviour<Event> {
            new(OneShotBehaviour { inner, action })
        }

        #[unsafe(no_mangle)]
        pub extern "C" fn behaviour_oneshot_free(oneshot: *mut OneShotBehaviour<Event>) {
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

        pub struct CyclicBehaviour<E> {
            /// Type value defined by the user implementing the trait.
            pub(in crate::ffi::behaviour) inner: *mut c_void,
            /// Action to be performed.
            pub(in crate::ffi::behaviour) action:
                extern "C" fn(*mut c_void, *mut Context<E>, *mut AgentState),
            /// Whether the behaviour has finished.
            pub(in crate::ffi::behaviour) is_finished: extern "C" fn(*mut c_void) -> bool,
        }

        impl<E> CyclicBehaviourTrait for CyclicBehaviour<E> {
            type AgentState = AgentState;

            type Event = E;

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
        ) -> *mut CyclicBehaviour<Event> {
            new(CyclicBehaviour {
                inner,
                action,
                is_finished,
            })
        }

        #[unsafe(no_mangle)]
        pub extern "C" fn behaviour_cyclic_free(cyclic: *mut CyclicBehaviour<Event>) {
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

        pub struct TickerBehaviour<E> {
            /// Type value defined by the user implementing the trait.
            pub(in crate::ffi::behaviour) inner: *mut c_void,
            /// Action to be performed.
            pub(in crate::ffi::behaviour) action:
                extern "C" fn(*mut c_void, *mut Context<E>, *mut AgentState),
            /// Whether the behaviour has finished.
            pub(in crate::ffi::behaviour) is_finished: extern "C" fn(*mut c_void) -> bool,
            /// Interval in miliseconds until the next scheduled action.
            pub(in crate::ffi::behaviour) interval: extern "C" fn(*mut c_void) -> u64,
        }

        impl<E> TickerBehaviourTrait for TickerBehaviour<E> {
            type AgentState = AgentState;

            type Event = E;

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
        ) -> *mut TickerBehaviour<Event> {
            new(TickerBehaviour {
                inner,
                interval,
                action,
                is_finished,
            })
        }

        #[unsafe(no_mangle)]
        pub extern "C" fn behaviour_ticker_free(ticker: *mut TickerBehaviour<Event>) {
            non_null_or_bail!(ticker, "attemted to free ticker behaviour null-pointer");
            unsafe { drop_raw(ticker) };
        }
    }
}

pub(super) mod complex {
    pub(in crate::ffi) use self::fsm::FsmBehaviour;
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
        use super::{AgentState, Event, FsmBehaviour};
        use super::{drop_raw, from_raw, new, ref_from_raw};

        pub struct BehaviourVec<E>(Vec<Box<dyn Behaviour<AgentState = AgentState, Event = E>>>);

        impl<E> BehaviourVec<E> {
            fn new() -> Self {
                BehaviourVec(Vec::new())
            }

            fn add_behaviour<K>(
                &mut self,
                behaviour: impl IntoBehaviour<'static, K, AgentState = AgentState, Event = E>,
            ) {
                self.0.push(behaviour.into_behaviour());
            }
        }

        impl<E> IntoIterator for BehaviourVec<E> {
            type Item = Box<dyn Behaviour<AgentState = AgentState, Event = E>>;

            type IntoIter = alloc::vec::IntoIter<Self::Item>;

            fn into_iter(self) -> Self::IntoIter {
                self.0.into_iter()
            }
        }

        #[unsafe(no_mangle)]
        pub extern "C" fn behaviour_vec_new() -> *mut BehaviourVec<Event> {
            new(BehaviourVec::new())
        }

        #[unsafe(no_mangle)]
        pub extern "C" fn behaviour_vec_add_behaviour_oneshot(
            behaviour_vec: *mut BehaviourVec<Event>,
            oneshot: *mut OneShotBehaviour<Event>,
        ) {
            non_null!(behaviour_vec, "got sequential behaviour vec null-pointer");
            non_null!(oneshot, "got oneshot behaviour null-pointer");
            let behaviour_vec = unsafe { ref_from_raw(behaviour_vec) };
            let behaviour = unsafe { from_raw(oneshot) };
            behaviour_vec.add_behaviour(behaviour);
        }

        #[unsafe(no_mangle)]
        pub extern "C" fn behaviour_vec_add_behaviour_cyclic(
            behaviour_vec: *mut BehaviourVec<Event>,
            cyclic: *mut CyclicBehaviour<Event>,
        ) {
            non_null!(behaviour_vec, "got sequential behaviour vec null-pointer");
            non_null!(cyclic, "got cyclic behaviour null-pointer");
            let behaviour_vec = unsafe { ref_from_raw(behaviour_vec) };
            let behaviour = unsafe { from_raw(cyclic) };
            behaviour_vec.add_behaviour(behaviour);
        }

        #[unsafe(no_mangle)]
        pub extern "C" fn behaviour_vec_add_behaviour_ticker(
            behaviour_vec: *mut BehaviourVec<Event>,
            ticker: *mut TickerBehaviour<Event>,
        ) {
            non_null!(behaviour_vec, "got sequential behaviour vec null-pointer");
            non_null!(ticker, "got ticker behaviour null-pointer");
            let behaviour_vec = unsafe { ref_from_raw(behaviour_vec) };
            let behaviour = unsafe { from_raw(ticker) };
            behaviour_vec.add_behaviour(behaviour);
        }

        #[unsafe(no_mangle)]
        pub extern "C" fn behaviour_vec_add_behaviour_sequential(
            behaviour_vec: *mut BehaviourVec<Event>,
            sequential: *mut SequentialBehaviour<Event>,
        ) {
            non_null!(behaviour_vec, "got sequential behaviour vec null-pointer");
            non_null!(sequential, "got sequential behaviour null-pointer");
            let behaviour_vec = unsafe { ref_from_raw(behaviour_vec) };
            let behaviour = unsafe { from_raw(sequential) };
            behaviour_vec.add_behaviour(behaviour);
        }

        #[unsafe(no_mangle)]
        pub extern "C" fn behaviour_vec_add_behaviour_fsm(
            behaviour_vec: *mut BehaviourVec<Event>,
            fsm: *mut FsmBehaviour<Event>,
        ) {
            non_null!(behaviour_vec, "got sequential behaviour vec null-pointer");
            non_null!(fsm, "got fsm behaviour null-pointer");
            let behaviour_vec = unsafe { ref_from_raw(behaviour_vec) };
            let behaviour = unsafe { from_raw(fsm) };
            behaviour_vec.add_behaviour(behaviour);
        }

        #[unsafe(no_mangle)]
        pub extern "C" fn behaviour_vec_free(behaviour_vec: *mut BehaviourVec<Event>) {
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

        pub struct SequentialBehaviour<E> {
            /// Type value defined by the user implementing the trait.
            pub(in crate::ffi::behaviour) inner: *mut c_void,
            /// List of initial behaviours to be scheduled.
            pub(in crate::ffi::behaviour) initial_behaviours: Cell<*mut BehaviourVec<Event>>,
            /// Function to be executed for every event a child has emitted.
            pub(in crate::ffi::behaviour) handle_child_event:
                extern "C" fn(*mut c_void, *mut Event),
            /// Function to be executed after a child behaviour has performed its action.
            pub(in crate::ffi::behaviour) after_child_action:
                extern "C" fn(*mut c_void, *mut Context<E>, *mut AgentState),
        }

        impl<E> ComplexBehaviour for SequentialBehaviour<E> {
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
                (self.after_child_action)(
                    self.inner,
                    ptr::from_mut(ctx),
                    ptr::from_mut(agent_state),
                );
            }
        }

        impl<E: 'static> SequentialBehaviourTrait<'static> for SequentialBehaviour<E> {
            fn initial_behaviours(
                &self,
            ) -> impl IntoIterator<
                Item = Box<dyn Behaviour<AgentState = Self::AgentState, Event = Self::ChildEvent>>,
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
            initial_behaviours: *mut BehaviourVec<Event>,
            handle_child_event: extern "C" fn(*mut c_void, *mut Event),
            after_child_action: extern "C" fn(*mut c_void, *mut Context<Event>, *mut AgentState),
        ) -> *mut SequentialBehaviour<Event> {
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
        pub extern "C" fn behaviour_sequential_free(sequential: *mut SequentialBehaviour<Event>) {
            non_null_or_bail!(
                sequential,
                "attempted to free sequential behaviour null-pointer"
            );
            unsafe { drop_raw(sequential) };
        }
    }

    mod fsm;
}
