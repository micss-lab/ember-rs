use ember::behaviour::Context;

use super::event::Event;
use super::util::{from_raw, ref_from_raw};

// No `new` or `free` needed as this is a mutable borrow from rust.

#[unsafe(no_mangle)]
pub extern "C" fn context_emit_event(context: *mut Context<Event>, event: *mut Event) {
    non_null!(context, "got a context null-pointer");
    non_null!(event, "got a event null-pointer");
    let context = unsafe { ref_from_raw(context) };
    let event = unsafe { from_raw(event) };
    context.emit_event(event);
}

#[unsafe(no_mangle)]
pub extern "C" fn context_stop_container(context: *mut Context<Event>) {
    non_null!(context, "got a context null-pointer");
    let context = unsafe { ref_from_raw(context) };
    context.stop_container();
}

#[unsafe(no_mangle)]
pub extern "C" fn context_remove_agent(context: *mut Context<Event>) {
    non_null!(context, "got a context null-pointer");
    let context = unsafe { ref_from_raw(context) };
    context.remove_agent();
}

#[unsafe(no_mangle)]
pub extern "C" fn context_block_behaviour(context: *mut Context<Event>) {
    non_null!(context, "got a context null-pointer");
    let context = unsafe { ref_from_raw(context) };
    context.block_behaviour();
}
