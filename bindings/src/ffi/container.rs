use ember::{Agent, Container};

use crate::ffi::util::{drop_raw, ref_from_raw};

use super::agent_state::AgentState;
use super::event::Event;
use super::util::{from_raw, new};

/// Creates a new container instance.
///
/// # Safety
///
/// The ownership of the instance is transferred to the caller. Make sure to free the memory
/// with the accompanying [`container_free`].
#[unsafe(no_mangle)]
pub extern "C" fn container_new() -> *mut Container<'static, 'static> {
    log::trace!("Creating new container");
    new(Container::default())
}

#[unsafe(no_mangle)]
pub extern "C" fn container_free(container: *mut Container) {
    non_null_or_bail!(container, "attemted to free container null-pointer");
    unsafe { drop_raw(container) }
}

#[unsafe(no_mangle)]
pub extern "C" fn container_add_agent(
    container: *mut Container,
    agent: *mut Agent<'static, AgentState, Event>,
) {
    non_null!(container, "got container null-pointer");
    non_null!(agent, "got agent null-pointer");
    let agent = unsafe { from_raw(agent) };
    unsafe { (*container).add_agent(agent) };
}

#[unsafe(no_mangle)]
pub extern "C" fn container_start(container: *mut Container) -> i32 {
    non_null!(container, "got container null-pointer");
    let result = unsafe { from_raw(container) }.start();
    match result {
        Ok(()) => 0,
        Err(_) => 1,
    }
}

#[repr(C)]
pub struct ContainerPollResult {
    status: i32,
    should_stop: bool,
}

#[unsafe(no_mangle)]
pub extern "C" fn container_poll(container: *mut Container) -> ContainerPollResult {
    non_null!(container, "got container null-pointer");
    let container = unsafe { ref_from_raw(container) };
    let (should_stop, status) = match container.poll() {
        Ok(should_stop) => (should_stop, 0),
        Err(_) => (true, 1),
    };
    ContainerPollResult {
        status,
        should_stop,
    }
}
