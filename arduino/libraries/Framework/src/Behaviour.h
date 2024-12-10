#ifndef FRAMEWORK_BEHAVIOUR_H
#define FRAMEWORK_BEHAVIOUR_H

#include <functional>
#include <memory>

#include "FrameworkCore.h"
#include "Object.h"

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

template<class T, class S>
class OneShotBehaviour:
    public Behaviour,
    public Object<__ffi::OneShotBehaviour<S>> {
};

template<class T>
class OneShotBehaviour<T, __ffi::State>:
    public Behaviour,
    public Object<__ffi::OneShotBehaviour<__ffi::State>> {
  public:
    OneShotBehaviour():
        Object(__ffi::behaviour_oneshot_new([](__ffi::Context* context_, __ffi::State state) -> __ffi::State {
            Context context(context_);
            return T::action(context, state);
        }), __ffi::behaviour_oneshot_free) {}
};

template<class T>
class OneShotBehaviour<T, void>:
    public Behaviour,
    public Object<__ffi::OneShotBehaviour<void>> {
  public:
    OneShotBehaviour():
        Object(__ffi::behaviour_oneshot_new_void([](__ffi::Context* context_) -> void {
            Context context(context_);
            T::action(context);
        }), __ffi::behaviour_oneshot_free_void) {}

    virtual void __ffi_add_behaviour_to_agent(__ffi::Agent* agent) {
        __ffi::agent_add_behaviour_oneshot(
            agent,
            this->move_object()
        );
    }
};

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

#endif // FRAMEWORK_BEHAVIOUR_H
