#ifndef FRAMEWORK_BEHAVIOUR_CYCLICBEHAVIOUR_H
#define FRAMEWORK_BEHAVIOUR_CYCLICBEHAVIOUR_H

#include "Behaviour.h"

#include "../Object.h"

namespace framework {

namespace behaviour {

template<class Event=void>
class CyclicBehaviour:
    public Behaviour<Event>,
    public Object<__ffi::CyclicBehaviour> {
  public:
    CyclicBehaviour();
    virtual ~CyclicBehaviour();

    virtual void action(Context<Event>& context) = 0;
    virtual bool is_finished() const = 0;

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
CyclicBehaviour<Event>::CyclicBehaviour():
    Object(
        __ffi::behaviour_cyclic_new(
            this,
            [](void* self_, __ffi::Context<__ffi::Event>* context_) {
                CyclicBehaviour<Event>* self = static_cast<CyclicBehaviour<Event>*>(self_);
                Context<Event> context(context_);
                return self->action(context);
            },
            [](void* self_) -> bool {
                CyclicBehaviour<Event>* self = static_cast<CyclicBehaviour<Event>*>(self_);
                return self->is_finished();
            }
        ),
        __ffi::behaviour_cyclic_free
    ) {}

template<class Event>
CyclicBehaviour<Event>::~CyclicBehaviour() {}

template<class Event>
void CyclicBehaviour<Event>::__ffi_add_behaviour_to_agent(__ffi::Agent<__ffi::Event>* agent) {
    __ffi::agent_add_behaviour_cyclic(
        agent,
        this->move_object()
    );
}

template<class Event>
void CyclicBehaviour<Event>::__ffi_add_behaviour_to_context(
    __ffi::Context<__ffi::Event>* context,
    __ffi::ScheduleStrategy strategy
) {
    __ffi::context_insert_behaviour_cyclic(
        context,
        this->move_object(),
        strategy
    );
}

template<class Event>
void CyclicBehaviour<Event>::__ffi_add_behaviour_to_sequential_behaviour_queue(
    __ffi::SequentialBehaviourQueue<__ffi::Event>* queue
) {
    __ffi::behaviour_sequential_queue_add_behaviour_cyclic(
        queue,
        this->move_object()
    );
}

} // namespace behaviour

} // namespace framework

#endif // FRAMEWORK_BEHAVIOUR_CYCLICBEHAVIOUR_H
