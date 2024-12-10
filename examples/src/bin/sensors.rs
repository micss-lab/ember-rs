#![cfg_attr(target_os = "none", no_std)]
#![cfg_attr(target_os = "none", no_main)]

use no_std_framework_examples::setup_example;

setup_example!();

use core::time::Duration;

use no_std_framework_core::behaviour::{OneShotBehaviour, SequentialBehaviour, TickerBehaviour};
use no_std_framework_core::{Agent, Container};

fn example() {
    let container = Container::default().with_agent(
        Agent::new("sensor-agent")
            .with_behaviour(OneShotBehaviour::new(|_, _| {
                log::info!("Sensors and actuators have been configured");
                log::info!("Oneshot event ended");
            }))
            .with_behaviour(TickerBehaviour::new(
                Duration::from_secs(1),
                (),
                |_, state| {
                    log::info!("Broadcasting that this agent is alive.");
                    state
                },
            ))
            .with_behaviour(TickerBehaviour::new(
                Duration::from_millis(200),
                (),
                |_, state| {
                    log::info!("Reading sensor values...");
                    state
                },
            ))
            .with_behaviour(
                SequentialBehaviour::new(())
                    .with_behaviour(OneShotBehaviour::new(|_, _| {
                        log::info!("1 - Motors are turned on");
                    }))
                    .with_behaviour(OneShotBehaviour::new(|_, _| {
                        log::info!("2 - Motors are turned 90 degrees ");
                    }))
                    .with_behaviour(OneShotBehaviour::new(|_, _| {
                        log::info!("3 - Motors are turned off 3")
                    })),
            ),
    );
    container.start().unwrap();
}
