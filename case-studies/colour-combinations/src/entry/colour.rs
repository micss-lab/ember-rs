//! # Colour combination
//!
//! Colour combination agent maximizing the score of assembling a random sequence of colors with a
//! decision window of 2 colors.
//!
//! ## Scores table.
//!
//! | Colour 1 | Colour 2 | Score |
//! | ======== | ======== | ===== |
//! | Red      | Red      | 100   |
//! | Red      | OC(Any)  | 50    |
//! | OC(Any)  | Red      | 50    |
//! | OC(Same) | OC(Same) | 25    |
//! | OC(Any)  | OC(Any)  | 0     |
//!
//! ## Algorithm.
//!
//! 1. If the first arrived brick is red, it should press and grab it, then it should merge it with
//! the non-red second one to achieve 50 points.
//! 2. If the second brick is also red, it merges them to achieve a 100 score.
//! 3. If the first one is a non-red brick and the second one is a non-red brick, and they have the
//! same colours, they are pressed to obtain a 25 score.
//! 4. If the first one is a non-red brick and the second one is a non-red brick, and they do not
//! have the same colours, then the first one is ejected, and the second one is accepted into the
//! press platform.
//! 5. If the first one is a non-red brick and the second one is a red brick, then the first one is
//! ejected, and the second one is accepted into the press platform, assuming there can be a
//! red/red chance.

use alloc::boxed::Box;
use core::marker::PhantomData;

use no_std_framework_core::{
    behaviour::{
        fsm::{Fsm, FsmBehaviour, FsmEvent},
        Behaviour, BehaviourId, ComplexBehaviour, Context, CyclicBehaviour, IntoBehaviour,
        OneShotBehaviour,
    },
    Agent,
};

pub fn colour_agent(
    sequence: impl IntoIterator<Item = Colour> + 'static,
) -> Agent<ColourCombinatorState<impl Iterator<Item = Colour>>, ()> {
    Agent::new("colour", ColourCombinatorState::new(sequence.into_iter()))
        .with_behaviour(ColourCombinator::default())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Colour {
    Red,
    Green,
    Blue,
}

pub struct ColourCombinatorState<I> {
    iter: I,
    score: u32,
    next: Option<Colour>,
}

impl<I> ColourCombinatorState<I> {
    fn new(iter: I) -> Self {
        Self {
            iter,
            score: 0,
            next: None,
        }
    }

    fn made_combination(&mut self, c1: Colour, c2: Colour) {
        log::info!("Making combination between {:?} and {:?}.", c1, c2);

        let val = match (c1, c2) {
            (Colour::Red, Colour::Red) => 100,
            (Colour::Red, _) | (_, Colour::Red) => 50,
            (Colour::Green, Colour::Green) | (Colour::Blue, Colour::Blue) => 25,
            _ => 0,
        };
        self.score += val;
    }
}

impl<I> ColourCombinatorState<I>
where
    I: Iterator<Item = Colour>,
{
    fn next_window(&mut self) -> Option<(Colour, Option<Colour>)> {
        match self
            .next
            .take()
            .or_else(|| self.iter.next())
            .map(|c1| (c1, self.iter.next()))
        {
            r @ Some((_, Some(c2))) => {
                self.next = Some(c2);
                r
            }
            r => r,
        }
    }

    fn next(&mut self) -> Option<Colour> {
        self.next.take().or_else(|| self.iter.next())
    }
}

struct ColourCombinator<I>(PhantomData<I>);

impl<I> Default for ColourCombinator<I> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<I> ComplexBehaviour for ColourCombinator<I> {
    type Event = ();

    type ChildEvent = ();

    type AgentState = ColourCombinatorState<I>;
}

impl<I> FsmBehaviour for ColourCombinator<I>
where
    I: Iterator<Item = Colour> + 'static,
{
    type TransitionTrigger = Colour;

    fn fsm(&self) -> Fsm<Self::AgentState, Self::TransitionTrigger, Self::ChildEvent> {
        let (empty_id, empty) = behaviour_with_id(Empty::default());
        let (red_id, red) = behaviour_with_id(Red::default());
        let (blue_id, blue) = behaviour_with_id(Blue::default());
        let (green_id, green) = behaviour_with_id(Green::default());

        Fsm::builder()
            .with_behaviour(empty, true)
            .with_behaviour(red, false)
            .with_behaviour(blue, false)
            .with_behaviour(green, false)
            .with_transition(red_id, empty_id, None)
            .with_transition(blue_id, empty_id, None)
            .with_transition(green_id, empty_id, None)
            .with_transition(empty_id, red_id, Some(Colour::Red))
            .with_transition(empty_id, blue_id, Some(Colour::Blue))
            .with_transition(empty_id, green_id, Some(Colour::Green))
            .try_build(empty_id)
            .unwrap()
    }
}

struct Empty<I> {
    finsish: bool,
    _marker: PhantomData<I>,
}

impl<I> Default for Empty<I> {
    fn default() -> Self {
        Self {
            finsish: false,
            _marker: PhantomData,
        }
    }
}

impl<I> CyclicBehaviour for Empty<I>
where
    I: Iterator<Item = Colour>,
{
    type AgentState = ColourCombinatorState<I>;

    type Event = FsmEvent<Colour, ()>;

    fn action(&mut self, ctx: &mut Context<Self::Event>, state: &mut Self::AgentState) {
        let window = match state.next_window() {
            Some((c1, Some(c2))) => (c1, c2),
            Some((c1, None)) => {
                ctx.emit_event(FsmEvent::Trigger(c1));
                return;
            }
            None => {
                log::info!("Final score: {}", state.score);
                self.finsish = true;
                return;
            }
        };

        match window {
            (Colour::Red, _) => ctx.emit_event(FsmEvent::Trigger(Colour::Red)),
            (colour @ Colour::Green, Colour::Green) | (colour @ Colour::Blue, Colour::Blue) => {
                ctx.emit_event(FsmEvent::Trigger(colour))
            }
            (colour, _) => {
                log::info!("Ejecting brick {:?}", colour);
                return;
            }
        }
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

impl<I> OneShotBehaviour for Red<I>
where
    I: Iterator<Item = Colour>,
{
    type AgentState = ColourCombinatorState<I>;

    type Event = FsmEvent<Colour, ()>;

    fn action(&self, _: &mut Context<Self::Event>, state: &mut Self::AgentState) {
        log::info!("Storing red brick.");

        let colour: Option<Colour> = (|| loop {
            break match state.next_window()? {
                (Colour::Red, _) => Some(Colour::Red),
                (c, Some(Colour::Red)) => {
                    log::info!("Ejecting brick: {:?}", c);
                    continue;
                }
                (c, _) => Some(c),
            };
        })();

        let Some(colour) = colour else {
            log::warn!("Failed to make a final combination!");
            return;
        };

        state.made_combination(Colour::Red, colour);
    }
}

struct Green<I>(PhantomData<I>);

impl<I> Default for Green<I> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<I> OneShotBehaviour for Green<I>
where
    I: Iterator<Item = Colour>,
{
    type AgentState = ColourCombinatorState<I>;

    type Event = FsmEvent<Colour, ()>;

    fn action(&self, _: &mut Context<Self::Event>, state: &mut Self::AgentState) {
        log::info!("Storing green brick.");

        let Some(colour) = state.next() else {
            log::warn!("Failed to make a final combination!");
            return;
        };

        state.made_combination(Colour::Green, colour);
    }
}

struct Blue<I>(PhantomData<I>);

impl<I> Default for Blue<I> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<I> OneShotBehaviour for Blue<I>
where
    I: Iterator<Item = Colour>,
{
    type AgentState = ColourCombinatorState<I>;

    type Event = FsmEvent<Colour, ()>;

    fn action(&self, _: &mut Context<Self::Event>, state: &mut Self::AgentState) {
        log::info!("Storing blue brick.");

        let Some(colour) = state.next() else {
            log::warn!("Failed to make a final combination!");
            return;
        };

        state.made_combination(Colour::Blue, colour);
    }
}

fn behaviour_with_id<K, A: 'static, E: 'static>(
    behaviour: impl IntoBehaviour<K, AgentState = A, Event = E>,
) -> (BehaviourId, Box<dyn Behaviour<AgentState = A, Event = E>>) {
    let behaviour = behaviour.into_behaviour();
    (behaviour.id(), behaviour)
}
