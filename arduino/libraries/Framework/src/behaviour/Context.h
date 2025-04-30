#ifndef FRAMEWORK_BEHAVIOUR_CONTEXT_H
#define FRAMEWORK_BEHAVIOUR_CONTEXT_H

#include "../FrameworkCore.h"

#include "Event.h"

namespace framework {

namespace behaviour {

template<class E=void>
class Context {
  public:
    Context(__ffi::Context<__ffi::Event>* context);

  public:
    void emit_event(Event<E>&& event);

    void stop_container();
    void remove_agent();
    void block_behaviour();

  private:
    // Does not own the context value (essentially a mutable reference to the context).
    __ffi::Context<__ffi::Event>* context;
};

// ======================= Impl =======================

template<class E>
Context<E>::Context(__ffi::Context<__ffi::Event>* context):
    context(context) {}

template<class E>
void Context<E>::emit_event(Event<E>&& event) {
    __ffi::context_emit_event(this->context, event.move_object());
}

template<class E>
void Context<E>::stop_container() {
    __ffi::context_stop_container(this->context);
}

template<class E>
void Context<E>::remove_agent() {
    __ffi::context_remove_agent(this->context);
}

template<class E>
void Context<E>::block_behaviour() {
    __ffi::context_block_behaviour(this->context);
}

} // namespace behaviour

} // namespace framework

#endif // FRAMEWORK_BEHAVIOUR_CONTEXT_H
