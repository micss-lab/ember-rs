#![no_std]
#![no_main]

use esp_backtrace as _;
use esp_hal::prelude::*;

use no_std_framework::behaviour::{CyclicBehaviour, OneShotBehaviour, SimpleBehaviourState};
use no_std_framework::hardware::init_heap;
use no_std_framework::{Agent, Container};

const MESSAGE_AMOUNT: usize = 10;

#[entry]
fn main() -> ! {
    esp_println::logger::init_logger_from_env();
    init_heap();

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

    panic!("End of program");
}
