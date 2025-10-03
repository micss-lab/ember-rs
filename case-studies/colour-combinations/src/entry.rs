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

impl Colour {
    fn combine(self, other: Self) -> usize {
        match (self, other) {
            (Colour::Red, Colour::Red) => 100,
            (Colour::Red, _) | (_, Colour::Red) => 50,
            (Colour::Green, Colour::Green) | (Colour::Blue, Colour::Blue) => 25,
            _ => 0,
        }
    }
}

fn wrap_message(m: Message) -> MessageEnvelope {
    use ember::message::Receiver;
    let Receiver::Single(ref r) = m.receiver else {
        unimplemented!();
    };
    MessageEnvelope::new(r.clone(), m)
}

pub(crate) fn main() {
    let peripheral_start = esp_hal::time::now();

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

    log::debug!(
        "peripheral setup: {} ns",
        (esp_hal::time::now() - peripheral_start).to_nanos()
    );

    cfg_if::cfg_if! {
        if #[cfg(feature = "without-ember")] {
            let runtime_start = esp_hal::time::now();

            let sequence = SEQUENCE.map(Some).into_iter().chain(core::iter::once(None)).collect::<Vec<_>>();
            let mut windows = sequence.windows(2);
            let mut score = 0;
            let mut stored: Option<Colour> = None;
            while let Some(window) = windows.next() {
                match (stored.take(), window[0], window[1]) {
                    (s, Some(Colour::Red), _) => match s {
                        Some(stored) => score += stored.combine(Colour::Red),
                        None => stored = Some(Colour::Red),
                    },
                    (Some(s), Some(c1), _) => score += s.combine(c1),
                    (None, Some(_), Some(Colour::Red)) => {stored = Some(Colour::Red); let _ = windows.next(); },
                    (None, Some(c1), Some(c2)) if c1 == c2 => {score += c1.combine(c2); let _ = windows.next();}
                    (None, Some(c1), Some(c2)) if c1 != c2 => {stored = Some(c2); let _ = windows.next();},
                    _ => unreachable!(),
                }
            }

            log::info!("Final score: {}", score);

        } else {

            let ember_start = esp_hal::time::now();
            let belt = Belt::new(SEQUENCE);
            let container = Container::default()
                .with_agent(sort::sorting_agent(belt.clone()))
                .with_agent(build::builder_agent(belt.clone()))
                .with_agent(trash::trasher_agent(belt.clone()));

            log::debug!(
                "Ember setup: {} ns",
                (esp_hal::time::now() - ember_start).to_nanos()
            );

            let runtime_start = esp_hal::time::now();
            container.start().unwrap();
        }
    }

    log::debug!(
        "Runtime: {} ns",
        (esp_hal::time::now() - runtime_start).to_nanos()
    );
}
