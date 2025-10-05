use alloc::string::ToString;
use alloc::vec::Vec;
use core::ffi::{CStr, c_char};
use core::str::FromStr;

use ember::Aid;
use ember::message::{Content, Message, Performative, Receiver};

use crate::ffi::util::drop_raw;

use super::util::new;

#[unsafe(no_mangle)]
pub extern "C" fn message_new(
    performative: c_char,
    receivers: *const *const c_char,
    receivers_len: usize,
    ontology: *const c_char,
    content: *mut u8,
    content_len: usize,
) -> *mut Message {
    let performative = performative_from_c_char(performative);
    let receiver = {
        let receivers = unsafe { core::slice::from_raw_parts(receivers, receivers_len) };
        match receivers_len {
            1 => Receiver::Single(unsafe { aid_from_c_str_pointer(receivers[0]) }),
            _ => Receiver::Multiple(
                receivers
                    .iter()
                    .map(|r| unsafe { aid_from_c_str_pointer(*r) })
                    .collect(),
            ),
        }
    };
    let ontology = (!ontology.is_null()).then(|| {
        unsafe { CStr::from_ptr(ontology) }
            .to_str()
            .expect("ontology should be valid utf-8")
            .to_string()
    });
    let content = Content::Bytes(unsafe { Vec::from_raw_parts(content, content_len, content_len) });

    new(Message {
        performative,
        sender: None,
        receiver,
        reply_to: None,
        ontology,
        content,
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn message_free(message: *mut Message) {
    non_null_or_bail!(message, "attemted to free message null-pointer");
    unsafe { drop_raw(message) }
}

fn performative_from_c_char(performative: c_char) -> Performative {
    match performative {
        0 => Performative::AcceptProposal,
        1 => Performative::Agree,
        2 => Performative::Cancel,
        3 => Performative::Cfp,
        4 => Performative::Confirm,
        5 => Performative::Disconfirm,
        6 => Performative::Failure,
        7 => Performative::Inform,
        8 => Performative::InformIf,
        9 => Performative::InformRef,
        10 => Performative::NotUnderstood,
        11 => Performative::Propose,
        12 => Performative::QueryIf,
        13 => Performative::QueryRef,
        14 => Performative::Refuse,
        15 => Performative::RejectProposal,
        16 => Performative::Request,
        17 => Performative::RequestWhen,
        18 => Performative::RequestWhenever,
        19 => Performative::Subscribe,
        20 => Performative::Proxy,
        21 => Performative::Propagate,
        22 => Performative::Unknown,
        _ => unreachable!("performative from ffi out of range"),
    }
}

unsafe fn aid_from_c_str_pointer(aid: *const u8) -> Aid {
    Aid::from_str(unsafe {
        CStr::from_ptr(aid)
            .to_str()
            .expect("aid string should be valid utf-8")
    })
    .expect("failed to parse string as aid")
}
