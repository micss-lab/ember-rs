#ifndef FRAMEWORK_BEHAVIOUR_CYCLICBEHAVIOUR_H
#define FRAMEWORK_BEHAVIOUR_CYCLICBEHAVIOUR_H

#include "Behaviour.h"

#include "../Object.h"

namespace framework {

namespace behaviour {

template<class Message=void>
class CyclicBehaviour:
    public Behaviour<Message>,
    public Object<__ffi::CyclicBehaviour> {
  public:
    CyclicBehaviour();
    virtual ~CyclicBehaviour();

    virtual void action(Context<Message>& context) = 0;
    virtual bool is_finished() const = 0;

  public:
    virtual void __ffi_add_behaviour_to_agent(__ffi::Agent<__ffi::Message>* agent) override;

    virtual void __ffi_add_behaviour_to_sequential_behaviour_queue(
        __ffi::SequentialBehaviourQueue<__ffi::Message>* queue
    ) override;
};

// ======================= Impl =======================

template<class Message>
CyclicBehaviour<Message>::CyclicBehaviour():
    Object(
        __ffi::behaviour_cyclic_new(
            this,
            [](void* self_, __ffi::Context<__ffi::Message>* context_) {
                CyclicBehaviour<Message>* self = static_cast<CyclicBehaviour<Message>*>(self_);
                Context context(context_);
                return self->action(context);
            },
            [](void* self_) -> bool {
                CyclicBehaviour<Message>* self = static_cast<CyclicBehaviour<Message>*>(self_);
                return self->is_finished();
            }
        ),
        __ffi::behaviour_cyclic_free
    ) {}

template<class Message>
CyclicBehaviour<Message>::~CyclicBehaviour() {}

template<class Message>
void CyclicBehaviour<Message>::__ffi_add_behaviour_to_agent(__ffi::Agent<__ffi::Message>* agent) {
    __ffi::agent_add_behaviour_cyclic(
        agent,
        this->move_object()
    );
}

template<class Message>
void CyclicBehaviour<Message>::__ffi_add_behaviour_to_sequential_behaviour_queue(
    __ffi::SequentialBehaviourQueue<__ffi::Message>* queue
) {
    __ffi::behaviour_sequential_queue_add_behaviour_cyclic(
        queue,
        this->move_object()
    );
}

} // namespace behaviour

} // namespace framework

#endif // FRAMEWORK_BEHAVIOUR_CYCLICBEHAVIOUR_H
