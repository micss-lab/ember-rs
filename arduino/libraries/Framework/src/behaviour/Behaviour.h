#ifndef FRAMEWORK_BEHAVIOUR_BEHAVIOUR_H
#define FRAMEWORK_BEHAVIOUR_BEHAVIOUR_H

#include <functional>
#include <memory>

#include "FrameworkCore.h"

namespace framework {

namespace behaviour {

using State = __ffi::State;
using SimpleState = __ffi::SimpleState;

typedef __ffi::State (*Action)(__ffi::Context*, __ffi::State);

class Context {
  public:
    inline Context(__ffi::Context* context):
        context(context) {}

  private:
    // Does not own the context value (essentially a mutable reference to the context).
    __ffi::Context* context;
};

class Behaviour {
  public:
    virtual void __ffi_add_behaviour_to_agent(__ffi::Agent* agent) = 0;
};

} // namespace behaviour

} // namespace framework

#endif // FRAMEWORK_BEHAVIOUR_BEHAVIOUR_H
