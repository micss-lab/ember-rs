use alloc::string::ToString;
use core::ffi::{CStr, c_char};

use ember::message::MessageFilter;

use crate::ffi::util::{drop_raw, new};

use super::performative_from_c_char;

#[unsafe(no_mangle)]
pub extern "C" fn message_filter_free(filter: *mut MessageFilter) {
    non_null_or_bail!(filter, "got message filter null-pointer");
    unsafe { drop_raw(filter) }
}

#[unsafe(no_mangle)]
pub extern "C" fn message_filter_all() -> *mut MessageFilter {
    new(MessageFilter::all())
}

#[unsafe(no_mangle)]
pub extern "C" fn message_filter_none() -> *mut MessageFilter {
    new(MessageFilter::none())
}

#[unsafe(no_mangle)]
pub extern "C" fn message_filter_performative(performative: c_char) -> *mut MessageFilter {
    new(MessageFilter::performative(performative_from_c_char(
        performative,
    )))
}

#[unsafe(no_mangle)]
pub extern "C" fn message_filter_ontology(ontology: *const c_char) -> *mut MessageFilter {
    let ontology = unsafe { CStr::from_ptr(ontology) }
        .to_str()
        .expect("ontology string should be valid utf-8")
        .to_string();
    new(MessageFilter::ontology(ontology))
}
