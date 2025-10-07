#ifndef EMBER_BEHAVIOUR_CONTEXT_H
#define EMBER_BEHAVIOUR_CONTEXT_H

#include <optional>

#include "../EmberCore.h"

#include "Event.h"
#include "./message/Message.h"
#include "./message/Filter.h"

namespace ember {

namespace behaviour {

template<class E=void>
class Context {
  private:
    using Message = message::Message;
    using MessageEnvelope = message::MessageEnvelope;
    using MessageFilter = message::MessageFilter;
    
  public:
    Context(__ffi::Context<__ffi::Event>* context);

  public:
    void emit_event(Event<E>&& event);

    void stop_container();
    void remove_agent();
    void block_behaviour();
    void send_message(MessageEnvelope&& message);

    std::optional<Message> receive_message();
    std::optional<Message> receive_message_with_filter(MessageFilter&& filter);

  private:
    // Does not own the context value (essentially a mutable reference to the context).
    __ffi::Context<__ffi::Event>* context;
};

// ======================= Impl =======================

template<class E>
Context<E>::Context(__ffi::Context<__ffi::Event>* context):
    context(context) {}

template<class E>
void Context<E>::emit_event(Event<E>&& event) {
    __ffi::context_emit_event(this->context, event.move_object());
}

template<class E>
void Context<E>::stop_container() {
    __ffi::context_stop_container(this->context);
}

template<class E>
void Context<E>::remove_agent() {
    __ffi::context_remove_agent(this->context);
}

template<class E>
void Context<E>::block_behaviour() {
    __ffi::context_block_behaviour(this->context);
}

template<class E>
void Context<E>::send_message(MessageEnvelope&& message) {
    __ffi::context_send_message(this->context, message.move_object());
}

template<class E>
std::optional<message::Message> Context<E>::receive_message() {
    __ffi::Message* message = __ffi::context_receive_message(this->context);
    if (message == nullptr) {
        return std::nullopt;
    }
    return std::optional(message);
}

template<class E>
std::optional<message::Message> Context<E>::receive_message_with_filter(
    message::MessageFilter&& filter
) {
    __ffi::Message* message = __ffi::context_receive_message_with_filter(
        this->context,
        filter.move_object()
    );
    if (message == nullptr) {
        return std::nullopt;
    }
    return std::optional(message);
}

} // namespace behaviour

} // namespace ember

#endif // EMBER_BEHAVIOUR_CONTEXT_H
