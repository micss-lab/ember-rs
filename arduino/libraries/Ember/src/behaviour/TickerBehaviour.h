#ifndef EMBER_BEHAVIOUR_TICKERBEHAVIOUR_H
#define EMBER_BEHAVIOUR_TICKERBEHAVIOUR_H

#include "Behaviour.h"

#include "Context.h"

#include "../Object.h"
#include "../Unit.h"

namespace ember {

namespace behaviour {

template<class AgentState=Unit, class Event=void>
class TickerBehaviour:
    public Behaviour<AgentState, Event>,
    public Object<__ffi::TickerBehaviour> {
  public:
    TickerBehaviour();
    virtual ~TickerBehaviour();
    
    virtual uint64_t interval_millis() const = 0;
    virtual void action(Context<Event>& context, AgentState& agent_state) = 0;
    virtual bool is_finished() const = 0;

  public:
    virtual void __ffi_add_behaviour_to_agent(__ffi::Agent<__ffi::AgentState, __ffi::Event>* agent) override;
    virtual void __ffi_add_behaviour_to_behaviour_vec(__ffi::BehaviourVec* vec) override;
};

// ======================= Impl =======================

template<class AgentState, class Event>
TickerBehaviour<AgentState, Event>::TickerBehaviour():
    Object(
        __ffi::behaviour_ticker_new(
            this,
            [](void* self_) -> uint64_t {
                TickerBehaviour<AgentState, Event>* self = static_cast<TickerBehaviour<AgentState, Event>*>(self_);
                return self->interval_millis();
            },
            [](void* self_, __ffi::Context<__ffi::Event>* context_, __ffi::AgentState* agent_state_) {
                TickerBehaviour<AgentState, Event>* self = static_cast<TickerBehaviour<AgentState, Event>*>(self_);
                Context<Event> context(context_);
                AgentState& agent_state = *static_cast<AgentState*>(agent_state_->inner);
                return self->action(context, agent_state);
            },
            [](void* self_) -> bool {
                TickerBehaviour<AgentState, Event>* self = static_cast<TickerBehaviour<AgentState, Event>*>(self_);
                return self->is_finished();
            }
        ),
        __ffi::behaviour_ticker_free
    ) {}

template<class AgentState, class Event>
TickerBehaviour<AgentState, Event>::~TickerBehaviour() {}

template<class AgentState, class Event>
void TickerBehaviour<AgentState, Event>::__ffi_add_behaviour_to_agent(__ffi::Agent<__ffi::AgentState, __ffi::Event>* agent) {
    __ffi::agent_add_behaviour_ticker(
        agent,
        this->move_object()
    );
}

template<class AgentState, class Event>
void TickerBehaviour<AgentState, Event>::__ffi_add_behaviour_to_behaviour_vec(__ffi::BehaviourVec* vec) {
    __ffi::behaviour_vec_add_behaviour_ticker(
        vec,
        this->move_object()
    );
}

} // namespace behaviour

} // namespace ember

#endif // EMBER_BEHAVIOUR_TICKERBEHAVIOUR_H
