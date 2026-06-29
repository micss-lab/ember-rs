#![cfg_attr(target_os = "none", no_std)]
#![cfg_attr(target_os = "none", no_main)]

use alloc::string::String;
use log::info;

use ember::Container;
use ember::agent::bdi::term::FromTerm;
use ember::agent::bdi::{bdi_actions, bdi_agent};

use ember_examples::setup_example;

setup_example!();

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, FromTerm)]
#[ember(transparent)]
pub struct Entity(String);

#[bdi_agent(
    asl = {
        // Initial Beliefs
        at(robot, base).
        connected(base, warehouse).
        connected(warehouse, factory).

        // Initial Goal
        !deliver(parts, factory).

        // Main delivery plan
        +!deliver(Item, Dest)
          <- !go_to(warehouse);
             pickup(Item);
             !go_to(Dest);
             dropoff(Item);
             .stop_platform().

        // Subgoal: Already at location
        +!go_to(Loc) : at(robot, Loc)
          <- .log("info", "Already at location.").

        // Subgoal: Adjacent move
        +!go_to(Loc) : at(robot, Current) & connected(Current, Loc)
          <- drive(Current, Loc);
             -at(robot, Current);
             +at(robot, Loc).

        // Subgoal: Multi-step move (using unification logic & recursion)
        +!go_to(Loc) : at(robot, Current) & connected(Current, Intermediate) & connected(Intermediate, Loc)
          <- .log("info", "Calculating multi-step path...");
             !go_to(Intermediate);
             !go_to(Loc).
    }
)]
struct LogisticsAgent;

#[bdi_actions]
impl LogisticsAgent {
    fn drive(&mut self, from: Entity, to: Entity) {
        info!("[ACTION] 🚚 Driving from {} to {}", from.0, to.0);
    }

    fn pickup(&mut self, item: Entity) {
        info!("[ACTION] 📦 Picking up {}", item.0);
    }

    fn dropoff(&mut self, item: Entity) {
        info!("[ACTION] 📥 Dropping off {}", item.0);
    }
}

fn example() {
    info!("🏭 Starting BDI Logistics Demo 🏭");
    info!("====================================================\n");

    Container::new()
        .with_agent(LogisticsAgent.into_agent())
        .start()
        .expect("container encountered an error");

    info!("\n====================================================");
    info!("🏭 BDI Logistics demo finished. 🏭");
}
