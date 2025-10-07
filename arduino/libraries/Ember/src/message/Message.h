#ifndef EMBER_MESSAGE_MESSAGE_H
#define EMBER_MESSAGE_MESSAGE_H

#include <vector>
#include <string>
#include <cstdint>

#include "Object.h"

#include "EmberCore.h"

namespace ember {

namespace message {

class MessageEnvelope:
    public Object<__ffi::MessageEnvelope> {};


enum class Performative {
    AcceptProposal = 0,
    Agree,
    Cancel,
    Cfp,
    Confirm,
    Disconfirm,
    Failure,
    Inform,
    InformIf,
    InformRef,
    NotUnderstood,
    Propose,
    QueryIf,
    QueryRef,
    Refuse,
    RejectProposal,
    Request,
    RequestWhen,
    RequestWhenever,
    Subscribe,
    Proxy,
    Propagate,
    Unknown,
};

class Message:
    public Object<__ffi::Message> {
  public:
    Message(
        Performative performative,
        const std::vector<const char*>& receivers,
        const char* ontology,
        const std::vector<uint8_t>& content
    );
    Message(__ffi::Message*);

    MessageEnvelope wrap_with_envelope() &&;
};

} // namespace message

} // namespace ember

#endif // EMBER_MESSAGE_MESSAGE_H
