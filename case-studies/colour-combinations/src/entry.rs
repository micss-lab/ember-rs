use belt::Belt;
use esp_backtrace as _;

use ember::{
    Container,
    message::{Message, MessageEnvelope},
};
use esp_hal::clock::CpuClock;

const HEAP_SIZE: usize = 72 * 1024;

const SEQUENCE: [Colour; 13] = [
    Colour::Blue,
    Colour::Red,
    Colour::Red,
    Colour::Green,
    Colour::Green,
    Colour::Blue,
    Colour::Green,
    Colour::Red,
    Colour::Blue,
    Colour::Red,
    Colour::Blue,
    Colour::Blue,
    Colour::Blue,
];

mod belt;
mod build;
mod sort;
mod trash;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Colour {
    Red,
    Green,
    Blue,
}

fn wrap_message(m: Message) -> MessageEnvelope {
    use ember::message::Receiver;
    let Receiver::Single(ref r) = m.receiver else {
        unimplemented!();
    };
    MessageEnvelope::new(r.clone(), m)
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

    let belt = Belt::new(SEQUENCE);
    Container::default()
        .with_agent(sort::sorting_agent(belt.clone()))
        .with_agent(build::builder_agent(belt.clone()))
        .with_agent(trash::trasher_agent(belt.clone()))
        .start()
        .unwrap()
}
