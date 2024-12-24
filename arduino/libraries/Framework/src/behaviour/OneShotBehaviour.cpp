#include "OneShotBehaviour.h"

using namespace framework::behaviour;

OneShotBehaviour::OneShotBehaviour():
    Object(
        __ffi::behaviour_oneshot_new(
            this,
            [](void* self_, __ffi::Context<__ffi::Message>* context_) {
                OneShotBehaviour* self = static_cast<OneShotBehaviour*>(self_);
                Context context(context_);
                return self->action(context);
            }
        ),
        __ffi::behaviour_oneshot_free
    ) {}

OneShotBehaviour::~OneShotBehaviour() {}

void OneShotBehaviour::__ffi_add_behaviour_to_agent(__ffi::Agent<__ffi::Message>* agent) {
    __ffi::agent_add_behaviour_oneshot(
        agent,
        this->move_object()
    );
}

void OneShotBehaviour::__ffi_add_behaviour_to_sequential_behaviour_queue(
    __ffi::SequentialBehaviourQueue<__ffi::Message>* queue
) {
    __ffi::behaviour_sequential_queue_add_behaviour_oneshot(
        queue,
        this->move_object()
    );
}
