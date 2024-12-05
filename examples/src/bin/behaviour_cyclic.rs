#![cfg_attr(target_os = "none", no_std)]
#![cfg_attr(target_os = "none", no_main)]

use no_std_framework_examples::setup_example;

setup_example!();

use no_std_framework_core::behaviour::{CyclicBehaviour, OneShotBehaviour, SimpleBehaviourState};
use no_std_framework_core::{Agent, Container};

const MESSAGE_AMOUNT: usize = 10;

fn example() {
    struct CyclicState(usize);

    impl SimpleBehaviourState for CyclicState {
        fn finished(&self) -> bool {
            self.0 == 0
        }
    }

    let container = Container::default().with_agent(
        Agent::new("messaging-agent")
            .with_behaviour(OneShotBehaviour::new(|_, _| {
                log::info!("This is the cyclic agent.");
                log::info!("I will print a message {MESSAGE_AMOUNT} times");
            }))
            .with_behaviour(CyclicBehaviour::new(
                CyclicState(MESSAGE_AMOUNT),
                |ctx, mut state| {
                    state.0 -= 1;
                    log::info!("Hello there!");
                    if state.finished() {
                        ctx.stop()
                    }
                    state
                },
            )),
    );
    container.start().unwrap();
}
