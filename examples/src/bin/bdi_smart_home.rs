#![cfg_attr(target_os = "none", no_std)]
#![cfg_attr(target_os = "none", no_main)]

use log::info;

use ember::Container;
use ember::agent::bdi::term::FromTerm;
use ember::agent::bdi::{bdi_actions, bdi_agent};

use ember_examples::setup_example;

setup_example!();

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, FromTerm)]
pub enum Room {
    LivingRoom,
    Bedroom,
}

#[bdi_agent(
    asl = {
        time(night).

        !simulate.

        +!simulate
          <- !enter_room(living_room);
             !leave_room(living_room);
             -time(night);
             +time(day);
             !enter_room(bedroom);
             .stop_platform().

        +!enter_room(Room)
          <- .log("info", "🚶 Simulating entering room...");
             +presence(Room).

        +!leave_room(Room)
          <- .log("info", "🚶 Simulating leaving room...");
             -presence(Room).

        // Reactive plans triggering on Belief Addition/Deletion
        +presence(Room) : time(night)
          <- turn_on_light(Room).

        +presence(Room) : time(day)
          <- .log("info", "☀️  Motion detected, but it's daytime. Lights stay off.").

        -presence(Room)
          <- turn_off_light(Room).
    }
)]
struct SmartHomeAgent;

#[bdi_actions]
impl SmartHomeAgent {
    fn turn_on_light(&mut self, room: Room) {
        info!("[ACTION] 💡 Turning ON light in {room:?}");
    }

    fn turn_off_light(&mut self, room: Room) {
        info!("[ACTION] 🌑 Turning OFF light in {room:?}");
    }
}

fn example() {
    info!("🏠 Starting BDI Smart Home Demo 🏠");
    info!("====================================================\n");

    Container::new()
        .with_agent(SmartHomeAgent.into_agent())
        .start()
        .expect("container encountered an error");

    info!("\n====================================================");
    info!("🏠 BDI Smart Home demo finished. 🏠");
}
