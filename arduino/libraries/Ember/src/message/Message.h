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

class ContentView {
  public:
    inline ContentView(const uint8_t* data, size_t size) : data(data), size_(size) {}
    inline ContentView(__ffi::ContentView content): data(content.data), size_(content.len) {}
    inline const uint8_t& operator[](size_t i) const { return data[i]; }
    inline size_t size() const { return this->size_; }
    inline const uint8_t* begin() const { return this->data; }
    inline const uint8_t* end() const { return this->data + this->size_; }
  public:
    uint8_t const* const data;
  private:
    size_t size_;
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

    ContentView get_content() const;

    MessageEnvelope wrap_with_envelope() &&;
};

} // namespace message

} // namespace ember

#endif // EMBER_MESSAGE_MESSAGE_H
