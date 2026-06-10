#![cfg_attr(target_os = "none", no_std)]
#![cfg_attr(target_os = "none", no_main)]

use alloc::string::{String, ToString};

use ember::agent::bdi::literal::IntoLiteral;
use log::info;

use ember::Container;
use ember::agent::bdi::context::Context;
use ember::agent::bdi::sensor::{Percept, Perceptor};
use ember::agent::bdi::term::reference::TermRef;
use ember::agent::bdi::term::{FromTerm, FromTermError};
use ember::agent::bdi::{bdi_actions, bdi_agent};

use ember_examples::setup_example;

setup_example!();

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Location(String);

impl FromTerm<'_> for Location {
    fn from_term(term: TermRef<'_>) -> Result<Self, FromTermError> {
        match term {
            TermRef::String(s) => Ok(Location(s.to_string())),
            TermRef::Literal { functor, .. } => Ok(Location(functor.0.clone())),
            _ => Err(FromTermError::InvalidType(Some("location"))),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Item(String);

impl FromTerm<'_> for Item {
    fn from_term(term: TermRef<'_>) -> Result<Self, FromTermError> {
        match term {
            TermRef::String(s) => Ok(Item(s.to_string())),
            TermRef::Literal { functor, .. } => Ok(Item(functor.0.clone())),
            _ => Err(FromTermError::InvalidType(Some("item"))),
        }
    }
}

struct Thermometer(/* Some sensor pin */);

impl Perceptor for Thermometer {
    type Percept = SensorReading;

    fn percept(&mut self) -> Option<Self::Percept> {
        Some(SensorReading { temperature: 0.0 })
    }
}

#[derive(IntoLiteral, Percept)]
struct SensorReading {
    temperature: f32,
}

#[bdi_agent(
    percept_type = SensorReading,
    asl = {
        at(agent, home).
        at(coffee_machine, kitchen).

        !make_coffee.

        +!make_coffee : at(agent, Loc) & at(coffee_machine, Loc) & have(coffee_beans)
          <- .log("info", "Enjoying a fresh cup of coffee!");
             +done.

        +!make_coffee
          <- !go_to(kitchen);
             !get_beans;
             !make_coffee.

        +!go_to(Dest) : at(agent, Dest)
          <- print_loc("Already at ", Dest).

        +!go_to(Dest) : at(agent, From)
          <- move_location(From, Dest);
             -at(agent, From);
             +at(agent, Dest).

        +!get_beans : have(coffee_beans)
          <- .log("info", "Found coffee beans in the pantry.").

        +!get_beans
          <- buy(coffee_beans);
             +have(coffee_beans).

        +done
          <- .stop_platform().
    }
)]
struct CoffeeAgent;

#[bdi_actions]
impl CoffeeAgent {
    fn print_msg(&mut self, msg: Item) {
        info!("[ACTION] {}", msg.0);
    }

    fn print_loc(&mut self, msg: Item, loc: Location) {
        info!("[ACTION] {} {:?}", msg.0, loc.0);
    }

    fn move_location(&mut self, from: Location, to: Location) {
        info!("[ACTION] 🏃 Moving from {:?} to {:?}", from.0, to.0);
    }

    fn buy(&mut self, item: Item) {
        info!("[ACTION] 🛒 Buying {:?}", item.0);
    }

    fn stop_action(&mut self, context: &mut Context<CoffeeAgentAction>) {
        info!("[ACTION] 🛑 Stopping platform...");
        context.stop_platform();
    }
}

fn example() {
    info!("☕ Starting BDI Agent demo: The Coffee Maker ☕");
    info!("====================================================\n");

    info!("🧠 Agent has 2 initial beliefs.");
    info!("   - at(agent, home)");
    info!("   - at(coffee_machine, kitchen)\n");

    info!("🎯 Initial Goal: !make_coffee\n");

    info!("🚀 Starting agent container...\n");

    Container::new()
        .with_agent(CoffeeAgent.into_agent().with_sensor(Thermometer()))
        .start()
        .expect("container encountered an error");

    info!("\n====================================================");
    info!("☕ BDI Agent demo finished. ☕");
}
