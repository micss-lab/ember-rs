#ifndef FRAMEWORK_BEHAVIOUR_BEHAVIOUR_H
#define FRAMEWORK_BEHAVIOUR_BEHAVIOUR_H

#include <functional>
#include <memory>

#include "../FrameworkCore.h"

namespace framework {

namespace behaviour {

template<class Event=void> // used to store only behaviours who pass around the same event.
class Behaviour {
  public:
    virtual void __ffi_add_behaviour_to_agent(__ffi::Agent<__ffi::Event>* agent) = 0;
    virtual void __ffi_add_behaviour_to_context(
        __ffi::Context<__ffi::Event>* context,
        __ffi::ScheduleStrategy strategy
    ) = 0;
    virtual void __ffi_add_behaviour_to_sequential_behaviour_queue(
        __ffi::SequentialBehaviourQueue<__ffi::Event>* queue
    ) = 0;
};

} // namespace behaviour

} // namespace framework

#endif // FRAMEWORK_BEHAVIOUR_BEHAVIOUR_H
