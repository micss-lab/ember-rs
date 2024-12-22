#include "OneShotBehaviour.h"

using namespace framework::behaviour;

OneShotBehaviour::OneShotBehaviour():
    Object(
        __ffi::behaviour_oneshot_new(
            static_cast<void*>(this),
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
