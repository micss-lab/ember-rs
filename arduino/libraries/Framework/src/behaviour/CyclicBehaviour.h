#ifndef FRAMEWORK_BEHAVIOUR_CYCLICBEHAVIOUR_H
#define FRAMEWORK_BEHAVIOUR_CYCLICBEHAVIOUR_H

#include "Behaviour.h"

#include "../Object.h"

namespace framework {

namespace behaviour {

template<class T, class S, class P>
class CyclicBehaviour:
    public Behaviour,
    public Object<__ffi::CyclicBehaviour<__ffi::SimpleState, P>> {
};

template<class T, class S>
class CyclicBehaviour<T, S, __ffi::State>:
    public Behaviour,
    public Object<__ffi::CyclicBehaviour<__ffi::SimpleState, __ffi::State>> {
  public:
    CyclicBehaviour(std::unique_ptr<S> state):
        Object(__ffi::behaviour_cyclic_new(
            SimpleState { .value = static_cast<void*>(state.release()), .finished = false },
            [](__ffi::Context* context_, __ffi::State state) -> __ffi::State {
                Context context(context_);
                return T::action(context, state);
            }
        ), __ffi::behaviour_cyclic_free) {}
};

template<class T, class S>
class CyclicBehaviour<T, S, void>:
    public Behaviour,
    public Object<__ffi::CyclicBehaviour<__ffi::SimpleState, void>> {
  public:
    CyclicBehaviour(std::unique_ptr<S> state):
        Object(__ffi::behaviour_cyclic_new_void(
            SimpleState { .value = static_cast<void*>(state.release()), .finished = false },
            [](__ffi::Context* context_, __ffi::SimpleState* state) -> void {
                Context context(context_);
                T::action(context, *state);
            }
        ), __ffi::behaviour_cyclic_free_void) {}

    virtual void __ffi_add_behaviour_to_agent(__ffi::Agent* agent) {
        __ffi::agent_add_behaviour_cyclic(
            agent,
            this->move_object()
        );
    }
};

} // namespace behaviour

} // namespace framework

#endif // FRAMEWORK_BEHAVIOUR_CYCLICBEHAVIOUR_H
