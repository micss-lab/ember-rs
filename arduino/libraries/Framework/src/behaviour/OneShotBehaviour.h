#ifndef FRAMEWORK_BEHAVIOUR_ONESHOTBEHAVIOUR_H
#define FRAMEWORK_BEHAVIOUR_ONESHOTBEHAVIOUR_H

#include "Behaviour.h"

#include "../Object.h"

namespace framework {

namespace behaviour {

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

} // namespace behaviour

} // namespace framework

#endif // FRAMEWORK_BEHAVIOUR_ONESHOTBEHAVIOUR_H
