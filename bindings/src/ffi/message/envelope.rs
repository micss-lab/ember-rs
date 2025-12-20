use ember::message::MessageEnvelope;

use crate::ffi::util::drop_raw;

#[unsafe(no_mangle)]
pub unsafe extern "C" fn message_envelope_free(envelope: *mut MessageEnvelope) {
    non_null_or_bail!(envelope, "attempted to free message envelope null-pointer");
    unsafe { drop_raw(envelope) }
}

mod serde {
    use alloc::vec::Vec;
    use ember::message::MessageEnvelope;
    use ember_acc::serde::espnow::de::EspNowMessageDe;
    use ember_acc::serde::espnow::ser::EspNowMessageSer;

    use crate::ffi::util::{from_raw, new};

    #[repr(C)]
    pub struct PostcardBytes {
        pub data: *mut u8,
        pub len: usize,
        pub capacity: usize,
    }

    impl Drop for PostcardBytes {
        fn drop(&mut self) {
            unsafe { Vec::from_raw_parts(self.data, self.len, self.capacity) };
        }
    }

    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn message_envelope_serialize_to_postcard_bytes(
        envelope: *mut MessageEnvelope,
    ) -> PostcardBytes {
        non_null!(envelope, "got message envelope null-pointer");
        let envelope = unsafe { from_raw(envelope) };
        let (data, len, capacity) = {
            let mut bytes = postcard::to_allocvec(&EspNowMessageSer(&envelope))
                .expect("failed to serialize message envelope");
            let result = (bytes.as_mut_ptr(), bytes.len(), bytes.capacity());
            core::mem::forget(bytes);
            result
        };
        PostcardBytes {
            data,
            len,
            capacity,
        }
    }

    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn message_envelope_deserialize_from_postcard_bytes(
        data: *const u8,
        len: usize,
    ) -> *mut MessageEnvelope {
        non_null!(data, "got data null-pointer");
        let envelope = postcard::from_bytes::<EspNowMessageDe>(unsafe {
            core::slice::from_raw_parts(data, len)
        })
        .expect("failed to deserialize bytes into message envelope")
        .into_envelope();
        new(envelope)
    }

    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn postcard_bytes_free(_: PostcardBytes) {
        // Rust automatically calls drop impl.
    }
}
