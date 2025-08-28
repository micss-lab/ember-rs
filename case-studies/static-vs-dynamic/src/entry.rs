use alloc::boxed::Box;
use esp_backtrace as _;

use ember::{
    Container,
    behaviour::{Behaviour, Context, CyclicBehaviour, IntoBehaviour},
    core::agent::Agent,
};
use esp_hal::{clock::CpuClock, time::Instant};

const HEAP_SIZE: usize = 72 * 1024;

const COUNTER: usize = 600_000;

struct DynamicAgent {
    counter: Box<dyn Behaviour<Event = (), AgentState = ()>>,
}

impl DynamicAgent {
    fn new() -> Self {
        Self {
            counter: Counter::new(COUNTER).into_behaviour(),
        }
    }
}

impl Agent for DynamicAgent {
    fn update(&mut self, context: &mut ember::core::context::ContainerContext) -> bool {
        self.counter
            .action(&mut Context::new_using_container(context), &mut ())
    }

    fn get_name(&self) -> alloc::borrow::Cow<str> {
        alloc::borrow::Cow::Borrowed("dynamic")
    }
}

struct StaticAgent {
    counter: Counter,
}

impl StaticAgent {
    fn new() -> Self {
        Self {
            counter: Counter::new(COUNTER),
        }
    }
}

impl Agent for StaticAgent {
    fn update(&mut self, context: &mut ember::core::context::ContainerContext) -> bool {
        self.counter
            .action(&mut Context::new_using_container(context), &mut ());
        if self.counter.count == self.counter.target {
            log::info!(
                "[static-dispatch]: Counting to {} took {} ns",
                self.counter.target,
                (esp_hal::time::now() - self.counter.start).to_nanos()
            );
            context.should_stop = true;
            true
        } else {
            false
        }
    }

    fn get_name(&self) -> alloc::borrow::Cow<str> {
        alloc::borrow::Cow::Borrowed("static")
    }
}

struct Counter {
    target: usize,
    count: usize,
    start: Instant,
}

impl Counter {
    fn new(target: usize) -> Self {
        let start = esp_hal::time::now();
        Self {
            target,
            count: 0,
            start,
        }
    }
}

impl CyclicBehaviour for Counter {
    type AgentState = ();

    type Event = ();

    fn action(&mut self, _: &mut Context<Self::Event>, _: &mut Self::AgentState) {
        self.count += 1;
    }

    fn is_finished(&self) -> bool {
        if self.count == self.target {
            log::info!(
                "[dynamic-dispatch]: Counting to {} took {} ns",
                self.target,
                (esp_hal::time::now() - self.start).to_nanos()
            );
            panic!("finished!")
        } else {
            false
        }
    }
}

pub(crate) fn main() {
    // Set newline mode to linux line endings.
    esp_println::print!("\x1b[20h");
    esp_println::logger::init_logger_from_env();
    esp_alloc::heap_allocator!(HEAP_SIZE);

    log::info!("Running case study `colour-combinations`.");

    let _peripherals = esp_hal::init({
        let mut config = esp_hal::Config::default();
        config.cpu_clock = CpuClock::max();
        config
    });

    log::trace!("Initialized peripherals.");

    Container::default()
        .with_agent(StaticAgent::new())
        .start()
        .unwrap();
    Container::default()
        .with_agent(DynamicAgent::new())
        .start()
        .unwrap();
}
