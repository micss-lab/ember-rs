#![cfg_attr(target_os = "none", no_std)]
#![cfg_attr(target_os = "none", no_main)]

use ember_examples::setup_example;

setup_example!();

use alloc::boxed::Box;
use core::time::Duration;

use ember_core::behaviour::sequential::SequentialBehaviour;
use ember_core::behaviour::{
    Behaviour, ComplexBehaviour, Context, IntoBehaviour, OneShotBehaviour, TickerBehaviour,
};
use ember_core::{Agent, Container};

struct SensorInit;

impl OneShotBehaviour for SensorInit {
    type AgentState = ();

    type Event = ();

    fn action(&self, _: &mut Context<Self::Event>, _: &mut Self::AgentState) {
        log::info!("Sensors and actuators have been configured");
        log::info!("Oneshot event ended");
    }
}

struct AgentAliveBroadCast;

impl TickerBehaviour for AgentAliveBroadCast {
    type AgentState = ();

    type Event = ();

    fn interval(&self) -> Duration {
        Duration::from_secs(1)
    }

    fn action(&mut self, _: &mut Context<Self::Event>, _: &mut Self::AgentState) {
        log::info!("Broadcasting that this agent is alive.");
    }

    fn is_finished(&self) -> bool {
        false
    }
}

struct SensorValueReader;

impl TickerBehaviour for SensorValueReader {
    type AgentState = ();

    type Event = ();

    fn interval(&self) -> Duration {
        Duration::from_millis(200)
    }

    fn action(&mut self, _: &mut Context<Self::Event>, _: &mut Self::AgentState) {
        log::info!("Reading sensor values...");
    }

    fn is_finished(&self) -> bool {
        false
    }
}

struct MotorMovements;

impl ComplexBehaviour for MotorMovements {
    type AgentState = ();

    type Event = ();

    type ChildEvent = ();
}

impl SequentialBehaviour for MotorMovements {
    fn initial_behaviours(
        &self,
    ) -> impl IntoIterator<
        Item = Box<dyn Behaviour<AgentState = Self::AgentState, Event = Self::ChildEvent>>,
    > {
        struct MotorStartUp;

        impl OneShotBehaviour for MotorStartUp {
            type AgentState = ();

            type Event = ();

            fn action(&self, _: &mut Context<Self::Event>, _: &mut Self::AgentState) {
                log::info!("1 - Motors are turned on");
            }
        }

        struct MotorTurn;

        impl OneShotBehaviour for MotorTurn {
            type AgentState = ();

            type Event = ();

            fn action(&self, _: &mut Context<Self::Event>, _: &mut Self::AgentState) {
                log::info!("2 - Motors are turned 90 degrees ");
            }
        }

        struct MotorShutDown;

        impl OneShotBehaviour for MotorShutDown {
            type AgentState = ();

            type Event = ();

            fn action(&self, _: &mut Context<Self::Event>, _: &mut Self::AgentState) {
                log::info!("3 - Motors are turned off 3")
            }
        }

        [
            MotorStartUp.into_behaviour(),
            MotorTurn.into_behaviour(),
            MotorShutDown.into_behaviour(),
        ]
    }
}

fn example() {
    let container = Container::default().with_agent(
        Agent::new("sensor-agent", ())
            .with_behaviour(SensorInit)
            .with_behaviour(AgentAliveBroadCast)
            .with_behaviour(SensorValueReader)
            .with_behaviour(MotorMovements),
    );
    container.start().unwrap();
}
