#include "Message.h"

#include <cstring>

using namespace ember::message;

template<typename T>
T* copy_vector_to_leaked_array(const std::vector<T>& data) {
    T* result = new T[data.size()];
    std::memcpy(result, data.data(), data.size() * sizeof(T));
    return result;
}

Message::Message(
    Performative performative,
    const std::vector<const char*>& receivers,
    const char* ontology,
    const std::vector<uint8_t>& content
): Object(
    __ffi::message_new(
        static_cast<char>(performative),
        receivers.data(), receivers.size(),
        ontology,
        // The ownership of the leaked array is passed over the ffi.
        copy_vector_to_leaked_array<uint8_t>(content), content.size()
    ), __ffi::message_free
) {
}

Message::Message(__ffi::Message* inner): 
    Object(inner, __ffi::message_free) {}

MessageEnvelope Message::wrap_with_envelope() && {
    __ffi::message_wrap_with_envelope(this->move_object());
}
