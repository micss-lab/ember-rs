#ifndef FRAMEWORK_BEHAVIOUR_MESSAGE_H
#define FRAMEWORK_BEHAVIOUR_MESSAGE_H

#include <memory>

#include "FrameworkCore.h"

#include "../Object.h"

namespace framework {

namespace behaviour {

template<class M>
class Message:
    public Object<__ffi::Message> {
  public:
    Message(std::unique_ptr<M>&& message);

    M* value();

  public:
    Message(__ffi::Message* message);
};

template<class M>
Message<M>::Message(std::unique_ptr<M>&& message):
    Object(
        __ffi::message_new(message.release()),
        __ffi::message_free
    ) {}

template<class M>
M* Message<M>::value() {
    M* message = static_cast<M*>(this->object->inner);
    return message;
}

template<class M>
Message<M>::Message(__ffi::Message* message):
    Object(
        message,
        __ffi::message_free
    ) {}

} // namespace behaviour

} // namespace framework

#endif // FRAMEWORK_BEHAVIOUR_MESSAGE_H
