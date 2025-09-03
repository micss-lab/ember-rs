//! # Colour combination
//!
//! Colour combination agent maximizing the score of assembling a random sequence of colours with a
//! decision window of 2 colours.
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
//!    the non-red second one to achieve 50 points.
//! 2. If the second brick is also red, it merges them to achieve a 100 score.
//! 3. If the first one is a non-red brick and the second one is a non-red brick, and they have the
//!    same colours, they are pressed to obtain a 25 score.
//! 4. If the first one is a non-red brick and the second one is a non-red brick, and they do not
//!    have the same colours, then the first one is ejected, and the second one is accepted into the
//!    press platform.
//! 5. If the first one is a non-red brick and the second one is a red brick, then the first one is
//!    ejected, and the second one is accepted into the press platform, assuming there can be a
//!    red/red chance.

use ember::{
    Agent,
    behaviour::{
        ComplexBehaviour, Context, CyclicBehaviour, IntoBehaviourWithId, OneShotBehaviour,
        fsm::{Fsm, FsmBehaviour, FsmEvent},
    },
};

use super::{
    Colour,
    belt::{Belt, Window},
    build::BuildMessage,
    trash::TrashMessage,
    wrap_message,
};

pub fn sorting_agent(belt: Belt) -> Agent<'static, Belt, ()> {
    Agent::new("sorting", belt).with_behaviour(DecisionBehaviour)
}

struct DecisionBehaviour;

impl ComplexBehaviour for DecisionBehaviour {
    type Event = ();

    type ChildEvent = ();

    type AgentState = Belt;
}

impl FsmBehaviour<'_> for DecisionBehaviour {
    type TransitionTrigger = Colour;

    fn fsm(&self) -> Fsm<'static, Self::AgentState, Self::TransitionTrigger, Self::ChildEvent> {
        let (empty_id, empty) = Empty::default().into_behaviour_with_id();
        let (red_id, red) = Red::default().into_behaviour_with_id();
        let (blue_id, blue) = Blue.into_behaviour_with_id();
        let (green_id, green) = Green.into_behaviour_with_id();

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

struct Empty {
    finsish: bool,
}

impl Default for Empty {
    fn default() -> Self {
        Self { finsish: false }
    }
}

impl CyclicBehaviour for Empty {
    type AgentState = Belt;

    type Event = FsmEvent<Colour, ()>;

    fn action(&mut self, ctx: &mut Context<Self::Event>, state: &mut Self::AgentState) {
        let window = match state.next_window() {
            Some(Window {
                first,
                second: Some(second),
            }) => (first, second),
            Some(Window { first, .. }) => {
                build(&mut *ctx);
                ctx.emit_event(FsmEvent::Trigger(first));
                return;
            }
            None => {
                state.print_score();
                self.finsish = true;
                ctx.stop_container();
                return;
            }
        };

        match window {
            (Colour::Red, _) => {
                build(&mut *ctx);
                ctx.emit_event(FsmEvent::Trigger(Colour::Red));
            }
            (colour @ Colour::Green, Colour::Green) | (colour @ Colour::Blue, Colour::Blue) => {
                build(&mut *ctx);
                ctx.emit_event(FsmEvent::Trigger(colour))
            }
            (_, _) => {
                trash(&mut *ctx);
            }
        }
    }

    fn is_finished(&self) -> bool {
        self.finsish
    }
}

#[derive(Default)]
struct Red {
    /// This state can decide to trash multiple products, so keep track of when to finish.
    built: bool,
}

impl CyclicBehaviour for Red {
    type AgentState = Belt;

    type Event = FsmEvent<Colour, ()>;

    fn action(&mut self, ctx: &mut Context<Self::Event>, state: &mut Self::AgentState) {
        match state.next_window() {
            Some(Window {
                first: Colour::Red, ..
            }) => {
                build(ctx);
                self.built = true;
            }
            Some(Window {
                first: _,
                second: Some(Colour::Red),
            }) => trash(ctx),
            Some(_) => {
                build(ctx);
                self.built = true;
            }
            None => log::warn!("Failed to make final combination!"),
        }
    }

    fn is_finished(&self) -> bool {
        self.built
    }
}

struct Green;

impl OneShotBehaviour for Green {
    type AgentState = Belt;

    type Event = FsmEvent<Colour, ()>;

    fn action(&self, ctx: &mut Context<Self::Event>, state: &mut Self::AgentState) {
        green_blue_action(state.peek_next(), ctx);
    }
}

struct Blue;

impl OneShotBehaviour for Blue {
    type AgentState = Belt;

    type Event = FsmEvent<Colour, ()>;

    fn action(&self, ctx: &mut Context<Self::Event>, state: &mut Self::AgentState) {
        green_blue_action(state.peek_next(), ctx);
    }
}

fn green_blue_action<E>(next: Option<Colour>, ctx: &mut Context<E>) {
    match next {
        Some(_) => build(ctx),
        None => log::warn!("Failed to make final combination!"),
    }
}

fn build<E>(ctx: &mut Context<E>) {
    ctx.send_message(wrap_message(BuildMessage.into_message()));
}

fn trash<E>(ctx: &mut Context<E>) {
    ctx.send_message(wrap_message(TrashMessage.into_message()));
}
