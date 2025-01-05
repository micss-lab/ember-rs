#ifndef FRAMEWORK_BEHAVIOUR_TICKERBEHAVIOUR_H
#define FRAMEWORK_BEHAVIOUR_TICKERBEHAVIOUR_H

#include "Behaviour.h"

#include "../Object.h"

namespace framework {

namespace behaviour {

template<class Message=void>
class TickerBehaviour:
    public Behaviour<Message>,
    public Object<__ffi::TickerBehaviour> {
  public:
    TickerBehaviour();
    virtual ~TickerBehaviour();
    
    virtual uint64_t interval_millis() const = 0;
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
TickerBehaviour<Message>::TickerBehaviour():
    Object(
        __ffi::behaviour_ticker_new(
            this,
            [](void* self_) -> uint64_t {
                TickerBehaviour<Message>* self = static_cast<TickerBehaviour<Message>*>(self_);
                return self->interval_millis();
            },
            [](void* self_, __ffi::Context<__ffi::Message>* context_) {
                TickerBehaviour<Message>* self = static_cast<TickerBehaviour<Message>*>(self_);
                Context<Message> context(context_);
                return self->action(context);
            },
            [](void* self_) -> bool {
                TickerBehaviour<Message>* self = static_cast<TickerBehaviour<Message>*>(self_);
                return self->is_finished();
            }
        ),
        __ffi::behaviour_ticker_free
    ) {}

template<class Message>
TickerBehaviour<Message>::~TickerBehaviour() {}

template<class Message>
void TickerBehaviour<Message>::__ffi_add_behaviour_to_agent(__ffi::Agent<__ffi::Message>* agent) {
    __ffi::agent_add_behaviour_ticker(
        agent,
        this->move_object()
    );
}

template<class Message>
void TickerBehaviour<Message>::__ffi_add_behaviour_to_sequential_behaviour_queue(
    __ffi::SequentialBehaviourQueue<__ffi::Message>* queue
) {
    __ffi::behaviour_sequential_queue_add_behaviour_ticker(
        queue,
        this->move_object()
    );
}

} // namespace behaviour

} // namespace framework

#endif // FRAMEWORK_BEHAVIOUR_TICKERBEHAVIOUR_H
