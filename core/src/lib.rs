#![no_std]

extern crate alloc;

#[cfg(not(target_os = "none"))]
extern crate std;

pub use self::agent::Agent;
pub use self::container::Container;

mod acl;
mod adt;
mod agent;
pub mod behaviour;
mod container;
mod context;
mod fipa;
mod util;
