#include "CyclicBehaviour.h"

using namespace framework::behaviour;

CyclicBehaviour::CyclicBehaviour():
    Object(
        __ffi::behaviour_cyclic_new(
            this,
            [](void* self_, __ffi::Context<__ffi::Message>* context_) {
                CyclicBehaviour* self = static_cast<CyclicBehaviour*>(self_);
                Context context(context_);
                return self->action(context);
            },
            [](void* self_) -> bool {
                CyclicBehaviour* self = static_cast<CyclicBehaviour*>(self_);
                return self->is_finished();
            }
        ),
        __ffi::behaviour_cyclic_free
    ) {}

CyclicBehaviour::~CyclicBehaviour() {}

void CyclicBehaviour::__ffi_add_behaviour_to_agent(__ffi::Agent<__ffi::Message>* agent) {
    __ffi::agent_add_behaviour_cyclic(
        agent,
        this->move_object()
    );
}

void CyclicBehaviour::__ffi_add_behaviour_to_sequential_behaviour_queue(
    __ffi::SequentialBehaviourQueue<__ffi::Message>* queue
) {
    __ffi::behaviour_sequential_queue_add_behaviour_cyclic(
        queue,
        this->move_object()
    );
}
