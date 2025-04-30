#ifndef FRAMEWORK_BEHAVIOUR_TICKERBEHAVIOUR_H
#define FRAMEWORK_BEHAVIOUR_TICKERBEHAVIOUR_H

#include "Behaviour.h"

#include "../Object.h"

namespace framework {

namespace behaviour {

template<class Event=void>
class TickerBehaviour:
    public Behaviour<Event>,
    public Object<__ffi::TickerBehaviour> {
  public:
    TickerBehaviour();
    virtual ~TickerBehaviour();
    
    virtual uint64_t interval_millis() const = 0;
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
TickerBehaviour<Event>::TickerBehaviour():
    Object(
        __ffi::behaviour_ticker_new(
            this,
            [](void* self_) -> uint64_t {
                TickerBehaviour<Event>* self = static_cast<TickerBehaviour<Event>*>(self_);
                return self->interval_millis();
            },
            [](void* self_, __ffi::Context<__ffi::Event>* context_) {
                TickerBehaviour<Event>* self = static_cast<TickerBehaviour<Event>*>(self_);
                Context<Event> context(context_);
                return self->action(context);
            },
            [](void* self_) -> bool {
                TickerBehaviour<Event>* self = static_cast<TickerBehaviour<Event>*>(self_);
                return self->is_finished();
            }
        ),
        __ffi::behaviour_ticker_free
    ) {}

template<class Event>
TickerBehaviour<Event>::~TickerBehaviour() {}

template<class Event>
void TickerBehaviour<Event>::__ffi_add_behaviour_to_agent(__ffi::Agent<__ffi::Event>* agent) {
    __ffi::agent_add_behaviour_ticker(
        agent,
        this->move_object()
    );
}

template<class Event>
void TickerBehaviour<Event>::__ffi_add_behaviour_to_context(
    __ffi::Context<__ffi::Event>* context,
    __ffi::ScheduleStrategy strategy
) {
    __ffi::context_insert_behaviour_ticker(
        context,
        this->move_object(),
        strategy
    );
}

template<class Event>
void TickerBehaviour<Event>::__ffi_add_behaviour_to_sequential_behaviour_queue(
    __ffi::SequentialBehaviourQueue<__ffi::Event>* queue
) {
    __ffi::behaviour_sequential_queue_add_behaviour_ticker(
        queue,
        this->move_object()
    );
}

} // namespace behaviour

} // namespace framework

#endif // FRAMEWORK_BEHAVIOUR_TICKERBEHAVIOUR_H
