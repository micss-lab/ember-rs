use core::marker::PhantomData;

use alloc::boxed::Box;

use no_std_framework_core::{
    behaviour::{
        fsm::{Fsm, FsmBehaviour, FsmEvent},
        Behaviour, BehaviourId, ComplexBehaviour, Context, CyclicBehaviour, IntoBehaviour,
        OneShotBehaviour,
    },
    Agent,
};

pub fn colour_agent<I>(sequence: impl IntoIterator<IntoIter = I>) -> Agent<Sequence<I>, ()>
where
    I: Iterator<Item = Colour> + 'static,
{
    Agent::new("colour", Sequence(sequence.into_iter())).with_behaviour(ColourCombinator::default())
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub enum Colour {
    Red,
    Green,
    Blue,
}

pub struct Sequence<I>(I);

struct ColourCombinator<I>(PhantomData<I>);

impl<I> Default for ColourCombinator<I> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<I> ComplexBehaviour for ColourCombinator<I> {
    type Event = ();

    type ChildEvent = ();

    type AgentState = Sequence<I>;
}

impl<I> FsmBehaviour for ColourCombinator<I>
where
    I: Iterator<Item = Colour> + 'static,
{
    type TransitionTrigger = Colour;

    fn fsm(&self) -> Fsm<Self::AgentState, Self::TransitionTrigger, Self::ChildEvent> {
        let (read_id, read) = behaviour_with_id(Read::default());
        let (red_id, red) = behaviour_with_id(Red::default());
        let (blue_id, blue) = behaviour_with_id(Blue::default());
        let (green_id, green) = behaviour_with_id(Green::default());

        Fsm::builder()
            .with_behaviour(read, true)
            .with_behaviour(red, false)
            .with_behaviour(blue, false)
            .with_behaviour(green, false)
            .with_transition(red_id, read_id, None)
            .with_transition(blue_id, read_id, None)
            .with_transition(green_id, read_id, None)
            .with_transition(read_id, red_id, Some(Colour::Red))
            .with_transition(read_id, blue_id, Some(Colour::Blue))
            .with_transition(read_id, green_id, Some(Colour::Green))
            .try_build(read_id)
            .unwrap()
    }
}

struct Read<I> {
    finsish: bool,
    _marker: PhantomData<I>,
}

impl<I> Default for Read<I> {
    fn default() -> Self {
        Self {
            finsish: false,
            _marker: PhantomData,
        }
    }
}

impl<I> CyclicBehaviour for Read<I>
where
    I: Iterator<Item = Colour>,
{
    type AgentState = Sequence<I>;

    type Event = FsmEvent<Colour, ()>;

    fn action(&mut self, ctx: &mut Context<Self::Event>, state: &mut Self::AgentState) {
        let Some(colour) = state.0.next() else {
            self.finsish = true;
            return;
        };

        ctx.emit_event(FsmEvent::Trigger(colour));
    }

    fn is_finished(&self) -> bool {
        self.finsish
    }
}

struct Red<I>(PhantomData<I>);

impl<I> Default for Red<I> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<I> OneShotBehaviour for Red<I> {
    type AgentState = Sequence<I>;

    type Event = FsmEvent<Colour, ()>;

    fn action(&self, _: &mut Context<Self::Event>, _: &mut Self::AgentState) {
        log::info!("Red State");
    }
}

struct Green<I>(PhantomData<I>);

impl<I> Default for Green<I> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<I> OneShotBehaviour for Green<I> {
    type AgentState = Sequence<I>;

    type Event = FsmEvent<Colour, ()>;

    fn action(&self, _: &mut Context<Self::Event>, _: &mut Self::AgentState) {
        log::info!("Green State");
    }
}

struct Blue<I>(PhantomData<I>);

impl<I> Default for Blue<I> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<I> OneShotBehaviour for Blue<I> {
    type AgentState = Sequence<I>;

    type Event = FsmEvent<Colour, ()>;

    fn action(&self, _: &mut Context<Self::Event>, _: &mut Self::AgentState) {
        log::info!("Blue State");
    }
}

fn behaviour_with_id<K, A: 'static, E: 'static>(
    behaviour: impl IntoBehaviour<K, AgentState = A, Event = E>,
) -> (BehaviourId, Box<dyn Behaviour<AgentState = A, Event = E>>) {
    let behaviour = behaviour.into_behaviour();
    (behaviour.id(), behaviour)
}
