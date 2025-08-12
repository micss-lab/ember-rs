use alloc::borrow::Cow;
use core::marker::PhantomData;

use ember::behaviour::{
    fsm::{Fsm, FsmBehaviour, FsmEvent},
    ComplexBehaviour, Context, CyclicBehaviour,
};

use super::util::behaviour_with_id;

pub struct ThresholdNotification<A>(PhantomData<A>);

impl<A> ThresholdNotification<A> {
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

pub trait ThresholdConfig {
    fn current(&self) -> f32;

    fn low(&self) -> f32;
    fn low_notification(&self, current: f32) -> Cow<'static, str>;

    fn high(&self) -> f32;
    fn high_notification(&self, current: f32) -> Cow<'static, str>;

    fn normalized_notification(&self, current: f32) -> Cow<'static, str>;
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub enum ThresholdEvent {
    High,
    Normalized,
    Low,
}

impl<A> ComplexBehaviour for ThresholdNotification<A> {
    type Event = ();

    type ChildEvent = ();

    type AgentState = A;
}

impl<A> FsmBehaviour for ThresholdNotification<A>
where
    A: ThresholdConfig + 'static,
{
    type TransitionTrigger = ThresholdEvent;

    fn fsm(&self) -> Fsm<Self::AgentState, Self::TransitionTrigger, Self::ChildEvent> {
        let (high_id, high) = behaviour_with_id(High(PhantomData));
        let (low_id, low) = behaviour_with_id(Low(PhantomData));
        let (normal_id, normal) = behaviour_with_id(Normal(PhantomData));

        Fsm::builder()
            .with_behaviour(high, false)
            .with_behaviour(low, false)
            .with_behaviour(normal, false)
            .with_transition(low_id, normal_id, Some(ThresholdEvent::Normalized))
            .with_transition(high_id, normal_id, Some(ThresholdEvent::Normalized))
            .with_transition(normal_id, low_id, Some(ThresholdEvent::Low))
            .with_transition(normal_id, high_id, Some(ThresholdEvent::High))
            .try_build(normal_id)
            .unwrap()
    }
}

struct High<A>(PhantomData<A>);

impl<A> CyclicBehaviour for High<A>
where
    A: ThresholdConfig + 'static,
{
    type AgentState = A;

    type Event = FsmEvent<ThresholdEvent, ()>;

    fn action(&mut self, ctx: &mut Context<Self::Event>, state: &mut Self::AgentState) {
        if state.current() >= state.high() {
            return;
        }

        log::info!("{}", state.normalized_notification(state.current()));
        ctx.emit_event(FsmEvent::Trigger(ThresholdEvent::Normalized));
    }

    fn is_finished(&self) -> bool {
        false
    }
}

struct Low<A>(PhantomData<A>);

impl<A> CyclicBehaviour for Low<A>
where
    A: ThresholdConfig + 'static,
{
    type AgentState = A;

    type Event = FsmEvent<ThresholdEvent, ()>;

    fn action(&mut self, ctx: &mut Context<Self::Event>, state: &mut Self::AgentState) {
        if state.current() <= state.low() {
            return;
        }

        log::info!("{}", state.normalized_notification(state.current()));
        ctx.emit_event(FsmEvent::Trigger(ThresholdEvent::Normalized));
    }

    fn is_finished(&self) -> bool {
        false
    }
}

struct Normal<A>(PhantomData<A>);

impl<A> CyclicBehaviour for Normal<A>
where
    A: ThresholdConfig,
{
    type AgentState = A;

    type Event = FsmEvent<ThresholdEvent, ()>;

    fn action(&mut self, ctx: &mut Context<Self::Event>, state: &mut Self::AgentState) {
        if state.current() <= state.low() {
            log::warn!("{}", state.low_notification(state.current()));
            ctx.emit_event(FsmEvent::Trigger(ThresholdEvent::Low));
        } else if state.current() >= state.high() {
            log::warn!("{}", state.high_notification(state.current()));
            ctx.emit_event(FsmEvent::Trigger(ThresholdEvent::High));
        }
    }

    fn is_finished(&self) -> bool {
        false
    }
}
