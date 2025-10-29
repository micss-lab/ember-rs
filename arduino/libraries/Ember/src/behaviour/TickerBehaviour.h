#ifndef EMBER_BEHAVIOUR_TICKERBEHAVIOUR_H
#define EMBER_BEHAVIOUR_TICKERBEHAVIOUR_H

#include "Behaviour.h"

#include "Context.h"

#include "../Object.h"
#include "../Unit.h"

namespace ember {

namespace behaviour {

template<class AgentState=Unit, class Event=Unit>
class TickerBehaviour:
    public Behaviour<AgentState, Event>,
    public Object<__ffi::TickerBehaviour<__ffi::Event>> {
  public:
    TickerBehaviour();
    virtual ~TickerBehaviour();
    
    virtual uint64_t interval_millis() const = 0;
    virtual void action(Context<Event>& context, AgentState& agent_state) = 0;
    virtual bool is_finished() const = 0;

  public:
    virtual void __ffi_add_behaviour_to_agent(__ffi::Agent<__ffi::AgentState, __ffi::Event>* agent) override;
    virtual void __ffi_add_behaviour_to_behaviour_vec(__ffi::BehaviourVec<__ffi::Event>* vec) override;
};

template<class AgentState, class E>
class TickerBehaviour<AgentState, FsmEvent<E>>:
    public Behaviour<AgentState, FsmEvent<E>>,
    public Object<__ffi::TickerBehaviour<__ffi::FsmEvent<const char*, __ffi::Event>>> {
  public:
    TickerBehaviour();
    virtual ~TickerBehaviour();
    
    virtual uint64_t interval_millis() const = 0;
    virtual void action(Context<FsmEvent<E>>& context, AgentState& agent_state) = 0;
    virtual bool is_finished() const = 0;

  public:
    virtual uint32_t __ffi_add_behaviour_to_fsm_builder(
        __ffi::FsmBuilder<__ffi::AgentState, const char*, __ffi::Event>* builder,
        bool is_final
    ) override;
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
void TickerBehaviour<AgentState, Event>::__ffi_add_behaviour_to_behaviour_vec(__ffi::BehaviourVec<__ffi::Event>* vec) {
    __ffi::behaviour_vec_add_behaviour_ticker(
        vec,
        this->move_object()
    );
}

// ======================= FsmEvent spec Impl =======================

template<class AgentState, class E>
TickerBehaviour<AgentState, FsmEvent<E>>::TickerBehaviour():
    Object(
        __ffi::behaviour_fsm_child_behaviour_ticker_new(
            this,
            [](void* self_) -> uint64_t {
                TickerBehaviour<AgentState, FsmEvent<E>>* self = static_cast<TickerBehaviour<AgentState, FsmEvent<E>>*>(self_);
                return self->interval_millis();
            },
            [](void* self_, __ffi::Context<__ffi::FsmEvent<const char*, __ffi::Event>>* context_, __ffi::AgentState* agent_state_) {
                TickerBehaviour<AgentState, FsmEvent<E>>* self = static_cast<TickerBehaviour<AgentState, FsmEvent<E>>*>(self_);
                Context<FsmEvent<E>> context(context_);
                AgentState& agent_state = *static_cast<AgentState*>(agent_state_->inner);
                return self->action(context, agent_state);
            },
            [](void* self_) -> bool {
                TickerBehaviour<AgentState, FsmEvent<E>>* self = static_cast<TickerBehaviour<AgentState, FsmEvent<E>>*>(self_);
                return self->is_finished();
            }
        ),
        __ffi::behaviour_fsm_child_behaviour_ticker_free
    ) {}

template<class AgentState, class E>
TickerBehaviour<AgentState, FsmEvent<E>>::~TickerBehaviour() {}

template<class AgentState, class E>
uint32_t TickerBehaviour<AgentState, FsmEvent<E>>::__ffi_add_behaviour_to_fsm_builder(
    __ffi::FsmBuilder<__ffi::AgentState, const char*, __ffi::Event>* builder,
    bool is_final
) {
    return __ffi::behaviour_fsm_builder_add_behaviour_ticker(builder, this->move_object(), is_final);
}

} // namespace behaviour

} // namespace ember

#endif // EMBER_BEHAVIOUR_TICKERBEHAVIOUR_H
