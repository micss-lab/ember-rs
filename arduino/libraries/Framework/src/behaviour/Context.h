#ifndef FRAMEWORK_BEHAVIOUR_CONTEXT_H
#define FRAMEWORK_BEHAVIOUR_CONTEXT_H

#include "../FrameworkCore.h"

#include "Message.h"

namespace framework {

namespace behaviour {

template<class M=void>
class Context {
  public:
    Context(__ffi::Context<__ffi::Message>* context);

  public:
    void message_parent(Message<M>&& message);

  private:
    // Does not own the context value (essentially a mutable reference to the context).
    __ffi::Context<__ffi::Message>* context;
};

// ======================= Impl =======================

template<class M>
Context<M>::Context(__ffi::Context<__ffi::Message>* context):
    context(context) {}

template<class M>
void Context<M>::message_parent(Message<M>&& message) {
    __ffi::context_message_parent(this->context, message.move_object());
}

} // namespace behaviour

} // namespace framework

#endif // FRAMEWORK_BEHAVIOUR_CONTEXT_H
