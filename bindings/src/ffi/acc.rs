use core::ffi::{c_char, c_void};

use alloc::string::ToString;
use ember::Aid;
use ember::channels::Acc;
use ember::message::MessageEnvelope;

use crate::ffi::util::{from_raw, new};

pub struct CustomAcc {
    /// Type value defined by the user implementing the trait.
    inner: *mut c_void,
    send: extern "C" fn(*mut c_void, *const c_char, *const MessageEnvelope) -> bool,
    receive: extern "C" fn(*mut c_void) -> *mut MessageEnvelope,
}

impl Acc for CustomAcc {
    fn send(&mut self, aid: &Aid, message: MessageEnvelope) -> Result<(), ()> {
        (self.send)(
            self.inner,
            aid.to_string().as_ptr(),
            core::ptr::from_ref(&message),
        )
        .then_some(())
        .ok_or(())
    }

    fn receive(&mut self) -> Option<MessageEnvelope> {
        let message = (self.receive)(self.inner);
        if message.is_null() {
            return None;
        }
        Some(unsafe { from_raw(message) })
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn acc_custom_acc_new(
    inner: *mut c_void,
    send: extern "C" fn(*mut c_void, *const c_char, *const MessageEnvelope) -> bool,
    receive: extern "C" fn(*mut c_void) -> *mut MessageEnvelope,
) -> *mut CustomAcc {
    new(CustomAcc {
        inner,
        send,
        receive,
    })
}
