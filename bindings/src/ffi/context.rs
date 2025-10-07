use alloc::borrow::Cow;

use ember::behaviour::Context;
use ember::message::{Message, MessageEnvelope, MessageFilter};

use crate::ffi::util::new;

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

#[unsafe(no_mangle)]
pub extern "C" fn context_send_message(
    context: *mut Context<Event>,
    message: *mut MessageEnvelope,
) {
    non_null!(context, "got a context null-pointer");
    non_null!(message, "got a message null-pointer");
    let context = unsafe { ref_from_raw(context) };
    let message = unsafe { from_raw(message) };
    context.send_message(message);
}

#[unsafe(no_mangle)]
pub extern "C" fn context_receive_message(context: *mut Context<Event>) -> *mut Message {
    non_null!(context, "got a context null-pointer");
    let context = unsafe { ref_from_raw(context) };
    match context.receive_message(None) {
        Some(message) => new(message),
        None => core::ptr::null_mut(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn context_receive_message_with_filter(
    context: *mut Context<Event>,
    filter: *mut MessageFilter,
) -> *mut Message {
    non_null!(context, "got a context null-pointer");
    non_null!(filter, "got message filter null-pointer");
    let context = unsafe { ref_from_raw(context) };
    let filter = unsafe { from_raw(filter) };

    match context.receive_message(Some(Cow::Owned(filter))) {
        Some(message) => new(message),
        None => core::ptr::null_mut(),
    }
}
