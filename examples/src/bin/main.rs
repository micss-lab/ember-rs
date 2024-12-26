#![cfg_attr(target_os = "none", no_std)]
#![cfg_attr(target_os = "none", no_main)]

use no_std_framework_examples::setup_example;

setup_example!();

use no_std_framework_core::{Agent, Container};

fn example() {
    let container = Container::default()
        .with_agent::<()>(Agent::new("agent-0"))
        .with_agent::<()>(Agent::new("agent-1"));
    container.start().unwrap();
}
