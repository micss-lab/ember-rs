#ifndef FRAMEWORK_BEHAVIOUR_BEHAVIOUR_H
#define FRAMEWORK_BEHAVIOUR_BEHAVIOUR_H

#include <functional>
#include <memory>

#include "FrameworkCore.h"

namespace framework {

namespace behaviour {

template<class Message=void> // used to store only behaviours who pass around the same message.
class Behaviour {
  public:
    virtual void __ffi_add_behaviour_to_agent(__ffi::Agent<__ffi::Message>* agent) = 0;

    virtual void __ffi_add_behaviour_to_sequential_behaviour_queue(
        __ffi::SequentialBehaviourQueue<__ffi::Message>* queue
    ) = 0;
};

} // namespace behaviour

} // namespace framework

#endif // FRAMEWORK_BEHAVIOUR_BEHAVIOUR_H
