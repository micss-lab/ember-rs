#ifndef FRAMEWORK_BEHAVIOUR_ONESHOTBEHAVIOUR_H
#define FRAMEWORK_BEHAVIOUR_ONESHOTBEHAVIOUR_H

#include "Behaviour.h"

#include "../Object.h"

namespace framework {

namespace behaviour {

template<class Event=void>
class OneShotBehaviour:
    public Behaviour<Event>,
    public Object<__ffi::OneShotBehaviour> {
  public:
    OneShotBehaviour();
    virtual ~OneShotBehaviour();

    virtual void action(Context<Event>& context) const = 0;

  public:
    virtual void __ffi_add_behaviour_to_agent(__ffi::Agent<__ffi::Event>* agent) override;
    virtual void __ffi_add_behaviour_to_context(
        __ffi::Context<__ffi::Event>* context,
        __ffi::ScheduleStrategy strategy
    ) override;
    virtual void __ffi_add_behaviour_to_sequential_behaviour_queue(
        __ffi::SequentialBehaviourQueue<__ffi::Event>* queue
    ) override;
};

// ======================= Impl =======================

template<class Event>
OneShotBehaviour<Event>::OneShotBehaviour():
    Object(
        __ffi::behaviour_oneshot_new(
            this,
            [](void* self_, __ffi::Context<__ffi::Event>* context_) {
                OneShotBehaviour<Event>* self = static_cast<OneShotBehaviour<Event>*>(self_);
                Context<Event> context(context_);
                return self->action(context);
            }
        ),
        __ffi::behaviour_oneshot_free
    ) {}

template<class Event>
OneShotBehaviour<Event>::~OneShotBehaviour() {}

template<class Event>
void OneShotBehaviour<Event>::__ffi_add_behaviour_to_agent(__ffi::Agent<__ffi::Event>* agent) {
    __ffi::agent_add_behaviour_oneshot(
        agent,
        this->move_object()
    );
}

template<class Event>
void OneShotBehaviour<Event>::__ffi_add_behaviour_to_context(
    __ffi::Context<__ffi::Event>* context,
    __ffi::ScheduleStrategy strategy
) {
    __ffi::context_insert_behaviour_oneshot(
        context,
        this->move_object(),
        strategy
    );
}

template<class Event>
void OneShotBehaviour<Event>::__ffi_add_behaviour_to_sequential_behaviour_queue(
    __ffi::SequentialBehaviourQueue<__ffi::Event>* queue
) {
    __ffi::behaviour_sequential_queue_add_behaviour_oneshot(
        queue,
        this->move_object()
    );
}

} // namespace behaviour

} // namespace framework

#endif // FRAMEWORK_BEHAVIOUR_ONESHOTBEHAVIOUR_H
