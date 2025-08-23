#![cfg_attr(target_os = "none", no_std)]
#![cfg_attr(target_os = "none", no_main)]

use ember::{
    Agent, Container,
    behaviour::{
        ComplexBehaviour, Context, IntoBehaviour, IntoBehaviourWithId, TickerBehaviour,
        fsm::{Fsm, FsmBehaviour, FsmEvent},
    },
};
use ember_examples::setup_example;

setup_example!();

struct TrafficLight;

impl ComplexBehaviour for TrafficLight {
    type Event = ();

    type ChildEvent = ();

    type AgentState = ();
}

impl FsmBehaviour<'static> for TrafficLight {
    type TransitionTrigger = Switch;

    fn fsm(&self) -> Fsm<'static, Self::AgentState, Self::TransitionTrigger, Self::ChildEvent> {
        let red_light = RedLight::default().into_behaviour();
        let orange_light = OrangeLight::default().into_behaviour();
        let (begin_state, green_light) = GreenLight::default().into_behaviour_with_id();

        Fsm::builder()
            .with_transition(red_light.id(), green_light.id(), Some(Switch))
            .with_transition(green_light.id(), orange_light.id(), Some(Switch))
            .with_transition(orange_light.id(), red_light.id(), Some(Switch))
            .with_behaviour(red_light, false)
            .with_behaviour(orange_light, false)
            .with_behaviour(green_light, false)
            .try_build(begin_state)
            .unwrap()
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
struct Switch;

#[derive(Default)]
struct RedLight {
    counter: u32,
}

impl TickerBehaviour for RedLight {
    type AgentState = ();

    type Event = FsmEvent<Switch, ()>;

    fn interval(&self) -> core::time::Duration {
        core::time::Duration::from_secs(1)
    }

    fn action(&mut self, ctx: &mut Context<Self::Event>, _: &mut Self::AgentState) {
        log::info!("Current light: red");

        self.counter += 1;
        if self.is_finished() {
            log::info!("Switching light");
            ctx.emit_event(FsmEvent::Trigger(Switch));
        }
    }

    fn is_finished(&self) -> bool {
        self.counter == 3
    }

    fn reset(&mut self) {
        self.counter = 0;
    }
}

#[derive(Default)]
struct OrangeLight {
    counter: u32,
}

impl TickerBehaviour for OrangeLight {
    type AgentState = ();

    type Event = FsmEvent<Switch, ()>;

    fn interval(&self) -> core::time::Duration {
        core::time::Duration::from_secs(1)
    }

    fn action(&mut self, ctx: &mut Context<Self::Event>, _: &mut Self::AgentState) {
        log::info!("Current light: orange");

        self.counter += 1;
        if self.is_finished() {
            log::info!("Switching light");
            ctx.emit_event(FsmEvent::Trigger(Switch));
        }
    }

    fn is_finished(&self) -> bool {
        self.counter == 2
    }

    fn reset(&mut self) {
        self.counter = 0;
    }
}

#[derive(Default)]
struct GreenLight {
    counter: u32,
}

impl TickerBehaviour for GreenLight {
    type AgentState = ();

    type Event = FsmEvent<Switch, ()>;

    fn interval(&self) -> core::time::Duration {
        core::time::Duration::from_secs(1)
    }

    fn action(&mut self, ctx: &mut Context<Self::Event>, _: &mut Self::AgentState) {
        log::info!("Current light: green");

        self.counter += 1;
        if self.is_finished() {
            log::info!("Switching light");
            ctx.emit_event(FsmEvent::Trigger(Switch));
        }
    }

    fn is_finished(&self) -> bool {
        self.counter == 4
    }

    fn reset(&mut self) {
        self.counter = 0;
    }
}

fn example() {
    let container = Container::default()
        .with_agent(Agent::new("traffic-light", ()).with_behaviour(TrafficLight));
    container.start().unwrap();
}
