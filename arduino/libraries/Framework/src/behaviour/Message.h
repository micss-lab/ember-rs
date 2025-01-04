#ifndef FRAMEWORK_BEHAVIOUR_MESSAGE_H
#define FRAMEWORK_BEHAVIOUR_MESSAGE_H

#include <memory>

#include "FrameworkCore.h"

#include "../Object.h"

namespace framework {

namespace behaviour {

class Message:
    public Object<__ffi::Message> {
  private:
    static std::unique_ptr<Message> __ffi_from_void(void* self);
};

} // namespace behaviour

} // namespace framework

#endif // FRAMEWORK_BEHAVIOUR_MESSAGE_H
