#ifndef FRAMEWORK_BEHAVIOUR_SEQUENTIALBEHAVIOUR_H
#define FRAMEWORK_BEHAVIOUR_SEQUENTIALBEHAVIOUR_H

#include <vector>
#include <memory>

#include "Behaviour.h"

#include "../Object.h"

namespace framework {

namespace behaviour {

class SequentialBehaviourQueue:
    public Object<__ffi::SequentialBehaviourQueue<__ffi::Message>> {
  public:
    SequentialBehaviourQueue();

    template<class Message>
    void add_behaviour(std::unique_ptr<Behaviour<Message>>&& behaviour);
};

template<class Message=void>
class SequentialBehaviour:
    public Behaviour<Message>,
    public Object<__ffi::SequentialBehaviour> {
  public:
    SequentialBehaviour(SequentialBehaviourQueue&& initial_behaviours);
    virtual ~SequentialBehaviour();

    virtual void after_child_action(Context<Message>& context);

  public:
    virtual void __ffi_add_behaviour_to_agent(__ffi::Agent<__ffi::Message>* agent) override;

    virtual void __ffi_add_behaviour_to_sequential_behaviour_queue(
        __ffi::SequentialBehaviourQueue<__ffi::Message>* queue
    ) override;
};

// ======================= Impl =======================

template<class Message>
void SequentialBehaviourQueue::add_behaviour(std::unique_ptr<Behaviour<Message>>&& behaviour_) {
    Behaviour<Message>* behaviour = behaviour_.release();
    behaviour->__ffi_add_behaviour_to_sequential_behaviour_queue(this->object);
}

template<class Message>
SequentialBehaviour<Message>::SequentialBehaviour(SequentialBehaviourQueue&& initial_behaviours):
    Object (
        __ffi::behaviour_sequential_new(
            this,
            initial_behaviours.move_object(),
            [](void* self_, __ffi::Context<__ffi::Message>* context_) {
                SequentialBehaviour<Message>* self = static_cast<SequentialBehaviour<Message>*>(self_);
                Context<Message> context(context_);
                self->after_child_action(context);
            }
        ),
        __ffi::behaviour_sequential_free
    ) {}

template<class Message>
SequentialBehaviour<Message>::~SequentialBehaviour() {}

template<class Message>
void SequentialBehaviour<Message>::after_child_action(Context<Message>& context) {
    // Does nothing.
}

template<class Message>
void SequentialBehaviour<Message>::__ffi_add_behaviour_to_agent(__ffi::Agent<__ffi::Message>* agent) {
    __ffi::agent_add_behaviour_sequential(
        agent,
        this->move_object()
    );
}

template<class Message>
void SequentialBehaviour<Message>::__ffi_add_behaviour_to_sequential_behaviour_queue(
    __ffi::SequentialBehaviourQueue<__ffi::Message>* queue
) {
    __ffi::behaviour_sequential_queue_add_behaviour_sequential(
        queue,
        this->move_object()
    );
}

} // namespace behaviour

} // namespace framework

#endif // FRAMEWORK_BEHAVIOUR_SEQUENTIALBEHAVIOUR_H
