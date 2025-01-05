#ifndef FRAMEWORK_BEHAVIOUR_SEQUENTIALBEHAVIOUR_H
#define FRAMEWORK_BEHAVIOUR_SEQUENTIALBEHAVIOUR_H

#include <vector>
#include <memory>

#include "Behaviour.h"

#include "../Object.h"

namespace framework {

namespace behaviour {

template<class M=void>
class SequentialBehaviourQueue:
    public Object<__ffi::SequentialBehaviourQueue<__ffi::Message>> {
  public:
    SequentialBehaviourQueue();

    void add_behaviour(std::unique_ptr<Behaviour<M>>&& behaviour);
};

template<class M=void, class ChildMessage=void>
class SequentialBehaviour:
    public Behaviour<M>,
    public Object<__ffi::SequentialBehaviour> {
  public:
    SequentialBehaviour(SequentialBehaviourQueue<ChildMessage>&& initial_behaviours);
    virtual ~SequentialBehaviour();

    virtual void handle_child_message(Message<ChildMessage>&& message);

    virtual void after_child_action(Context<M>& context);

  public:
    virtual void __ffi_add_behaviour_to_agent(__ffi::Agent<__ffi::Message>* agent) override;

    virtual void __ffi_add_behaviour_to_sequential_behaviour_queue(
        __ffi::SequentialBehaviourQueue<__ffi::Message>* queue
    ) override;
};

// ======================= Impl =======================

template<class M>
SequentialBehaviourQueue<M>::SequentialBehaviourQueue():
    Object(__ffi::behaviour_sequential_queue_new(), __ffi::behaviour_sequential_queue_free) {}

template<class M>
void SequentialBehaviourQueue<M>::add_behaviour(std::unique_ptr<Behaviour<M>>&& behaviour_) {
    Behaviour<M>* behaviour = behaviour_.release();
    behaviour->__ffi_add_behaviour_to_sequential_behaviour_queue(this->object);
}

template<class M, class ChildMessage>
SequentialBehaviour<M, ChildMessage>::SequentialBehaviour(SequentialBehaviourQueue<ChildMessage>&& initial_behaviours):
    Object (
        __ffi::behaviour_sequential_new(
            this,
            initial_behaviours.move_object(),
            [](void* self_, __ffi::Message* message_) {
                SequentialBehaviour<M, ChildMessage>* self = static_cast<SequentialBehaviour<M, ChildMessage>*>(self_);
                Message<ChildMessage> message(message_);
                self->handle_child_message(std::move(message));
            },
            [](void* self_, __ffi::Context<__ffi::Message>* context_) {
                SequentialBehaviour<M>* self = static_cast<SequentialBehaviour<M>*>(self_);
                Context<M> context(context_);
                self->after_child_action(context);
            }
        ),
        __ffi::behaviour_sequential_free
    ) {}

template<class M, class ChildMessage>
SequentialBehaviour<M, ChildMessage>::~SequentialBehaviour() {}

template<class M, class ChildMessage>
void SequentialBehaviour<M, ChildMessage>::handle_child_message(Message<ChildMessage>&&) {
    // Does nothing.
}

template<class M, class ChildMessage>
void SequentialBehaviour<M, ChildMessage>::after_child_action(Context<M>& context) {
    // Does nothing.
}

template<class M, class ChildMessage>
void SequentialBehaviour<M, ChildMessage>::__ffi_add_behaviour_to_agent(__ffi::Agent<__ffi::Message>* agent) {
    __ffi::agent_add_behaviour_sequential(
        agent,
        this->move_object()
    );
}

template<class M, class ChildMessage>
void SequentialBehaviour<M, ChildMessage>::__ffi_add_behaviour_to_sequential_behaviour_queue(
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
