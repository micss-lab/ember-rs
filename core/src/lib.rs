#![cfg_attr(target_os = "none", no_std)]

extern crate alloc;

use alloc::string::String;

use behaviour::Context;

pub use self::agent::Agent;
pub use self::container::Container;

mod agent;
pub mod behaviour;
mod container;

trait Actor: 'static {
    fn update(&mut self, context: &mut Context<()>);

    fn get_name(&self) -> String;
}
