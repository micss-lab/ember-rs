#ifndef FRAMEWORK_BEHAVIOUR_BEHAVIOUR_H
#define FRAMEWORK_BEHAVIOUR_BEHAVIOUR_H

#include <functional>
#include <memory>

#include "FrameworkCore.h"

namespace framework {

namespace behaviour {

class Context {
  public:
    inline Context(__ffi::Context<__ffi::Message>* context):
        context(context) {}

  private:
    // Does not own the context value (essentially a mutable reference to the context).
    __ffi::Context<__ffi::Message>* context;
};

class Behaviour {
  public:
    virtual void __ffi_add_behaviour_to_agent(__ffi::Agent<__ffi::Message>* agent) = 0;
};

} // namespace behaviour

} // namespace framework

#endif // FRAMEWORK_BEHAVIOUR_BEHAVIOUR_H
