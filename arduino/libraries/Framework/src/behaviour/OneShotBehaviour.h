#ifndef FRAMEWORK_BEHAVIOUR_ONESHOTBEHAVIOUR_H
#define FRAMEWORK_BEHAVIOUR_ONESHOTBEHAVIOUR_H

#include "Behaviour.h"

#include "../Object.h"

namespace framework {

namespace behaviour {

template<class Message=void>
class OneShotBehaviour:
    public Behaviour<Message>,
    public Object<__ffi::OneShotBehaviour> {
  public:
    OneShotBehaviour();
    virtual ~OneShotBehaviour();

    virtual void action(Context<Message>& context) const = 0;

  public:
    virtual void __ffi_add_behaviour_to_agent(__ffi::Agent<__ffi::Message>* agent) override;
    virtual void __ffi_add_behaviour_to_context(
        __ffi::Context<__ffi::Message>* context,
        __ffi::ScheduleStrategy strategy
    ) override;
    virtual void __ffi_add_behaviour_to_sequential_behaviour_queue(
        __ffi::SequentialBehaviourQueue<__ffi::Message>* queue
    ) override;
};

// ======================= Impl =======================

template<class Message>
OneShotBehaviour<Message>::OneShotBehaviour():
    Object(
        __ffi::behaviour_oneshot_new(
            this,
            [](void* self_, __ffi::Context<__ffi::Message>* context_) {
                OneShotBehaviour<Message>* self = static_cast<OneShotBehaviour<Message>*>(self_);
                Context<Message> context(context_);
                return self->action(context);
            }
        ),
        __ffi::behaviour_oneshot_free
    ) {}

template<class Message>
OneShotBehaviour<Message>::~OneShotBehaviour() {}

template<class Message>
void OneShotBehaviour<Message>::__ffi_add_behaviour_to_agent(__ffi::Agent<__ffi::Message>* agent) {
    __ffi::agent_add_behaviour_oneshot(
        agent,
        this->move_object()
    );
}

template<class Message>
void OneShotBehaviour<Message>::__ffi_add_behaviour_to_context(
    __ffi::Context<__ffi::Message>* context,
    __ffi::ScheduleStrategy strategy
) {
    __ffi::context_insert_behaviour_oneshot(
        context,
        this->move_object(),
        strategy
    );
}

template<class Message>
void OneShotBehaviour<Message>::__ffi_add_behaviour_to_sequential_behaviour_queue(
    __ffi::SequentialBehaviourQueue<__ffi::Message>* queue
) {
    __ffi::behaviour_sequential_queue_add_behaviour_oneshot(
        queue,
        this->move_object()
    );
}

} // namespace behaviour

} // namespace framework

#endif // FRAMEWORK_BEHAVIOUR_ONESHOTBEHAVIOUR_H
