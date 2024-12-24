#include "SequentialBehaviour.h"

using namespace framework::behaviour;

SequentialBehaviourQueue::SequentialBehaviourQueue():
    Object(__ffi::behaviour_sequential_queue_new(), __ffi::behaviour_sequential_queue_free) {}

void SequentialBehaviourQueue::add_behaviour(std::unique_ptr<Behaviour>&& behaviour_) {
    Behaviour* behaviour = behaviour_.release();
    behaviour->__ffi_add_behaviour_to_sequential_behaviour_queue(this->object);
}

SequentialBehaviour::SequentialBehaviour(SequentialBehaviourQueue&& initial_behaviours):
    Object (
        __ffi::behaviour_sequential_new(
            this,
            initial_behaviours.move_object(),
            [](void* self_, __ffi::Context<__ffi::Message>* context_) {
                SequentialBehaviour* self = static_cast<SequentialBehaviour*>(self_);
                Context context(context_);
                self->after_child_action(context);
            }
        ),
        __ffi::behaviour_sequential_free
    ) {}

SequentialBehaviour::~SequentialBehaviour() {}

void SequentialBehaviour::after_child_action(Context& context) {
    // Does nothing.
}

void SequentialBehaviour::__ffi_add_behaviour_to_agent(__ffi::Agent<__ffi::Message>* agent) {
    __ffi::agent_add_behaviour_sequential(
        agent,
        this->move_object()
    );
}

void SequentialBehaviour::__ffi_add_behaviour_to_sequential_behaviour_queue(
    __ffi::SequentialBehaviourQueue<__ffi::Message>* queue
) {
    __ffi::behaviour_sequential_queue_add_behaviour_sequential(
        queue,
        this->move_object()
    );
}
