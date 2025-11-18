use ember::message::MessageEnvelope;

use crate::ffi::util::drop_raw;

#[unsafe(no_mangle)]
pub unsafe extern "C" fn message_envelope_free(envelope: *mut MessageEnvelope) {
    non_null_or_bail!(envelope, "attempted to free message envelope null-pointer");
    unsafe { drop_raw(envelope) }
}