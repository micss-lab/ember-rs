#![cfg_attr(target_os = "none", no_std)]
#![cfg_attr(target_os = "none", no_main)]

use no_std_framework_examples::setup_example;

setup_example!();

use core::time::Duration;

use no_std_framework_core::behaviour::sequential::{SequentialBehaviour, SequentialBehaviourQueue};
use no_std_framework_core::behaviour::{Context, OneShotBehaviour, TickerBehaviour};
use no_std_framework_core::{Agent, Container};

struct SensorInit;

impl OneShotBehaviour for SensorInit {
    type Event = ();

    fn action(&self, _: &mut Context<Self::Event>) {
        log::info!("Sensors and actuators have been configured");
        log::info!("Oneshot event ended");
    }
}

struct AgentAliveBroadCast;

impl TickerBehaviour for AgentAliveBroadCast {
    type Event = ();

    fn interval(&self) -> Duration {
        Duration::from_secs(1)
    }

    fn action(&mut self, _: &mut Context<Self::Event>) {
        log::info!("Broadcasting that this agent is alive.");
    }

    fn is_finished(&self) -> bool {
        false
    }
}

struct SensorValueReader;

impl TickerBehaviour for SensorValueReader {
    type Event = ();

    fn interval(&self) -> Duration {
        Duration::from_millis(200)
    }

    fn action(&mut self, _: &mut Context<Self::Event>) {
        log::info!("Reading sensor values...");
    }

    fn is_finished(&self) -> bool {
        false
    }
}

struct MotorMovements;

impl SequentialBehaviour for MotorMovements {
    type Event = ();

    type ChildEvent = ();

    fn initial_behaviours(&self) -> SequentialBehaviourQueue<Self::ChildEvent> {
        struct MotorStartUp;

        impl OneShotBehaviour for MotorStartUp {
            type Event = ();

            fn action(&self, _: &mut Context<Self::Event>) {
                log::info!("1 - Motors are turned on");
            }
        }

        struct MotorTurn;

        impl OneShotBehaviour for MotorTurn {
            type Event = ();

            fn action(&self, _: &mut Context<Self::Event>) {
                log::info!("2 - Motors are turned 90 degrees ");
            }
        }

        struct MotorShutDown;

        impl OneShotBehaviour for MotorShutDown {
            type Event = ();

            fn action(&self, _: &mut Context<Self::Event>) {
                log::info!("3 - Motors are turned off 3")
            }
        }

        SequentialBehaviourQueue::new()
            .with_behaviour(MotorStartUp)
            .with_behaviour(MotorTurn)
            .with_behaviour(MotorShutDown)
    }
}

fn example() {
    let container = Container::default().with_agent(
        Agent::new("sensor-agent")
            .with_behaviour(SensorInit)
            .with_behaviour(AgentAliveBroadCast)
            .with_behaviour(SensorValueReader)
            .with_behaviour(MotorMovements),
    );
    container.start().unwrap();
}
