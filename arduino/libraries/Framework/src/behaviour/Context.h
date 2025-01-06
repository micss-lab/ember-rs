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

    void stop_container();
    void remove_agent();
    void block_behaviour();

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

template<class M>
void Context<M>::stop_container() {
    __ffi::context_stop_container(this->object);
}

template<class M>
void Context<M>::remove_agent() {
    __ffi::context_remove_agent(this->object);
}

template<class M>
void Context<M>::block_behaviour() {
    __ffi::context_block_behaviour(this->object);
}

} // namespace behaviour

} // namespace framework

#endif // FRAMEWORK_BEHAVIOUR_CONTEXT_H
