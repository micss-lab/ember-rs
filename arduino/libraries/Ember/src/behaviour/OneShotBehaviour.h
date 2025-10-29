#ifndef EMBER_BEHAVIOUR_ONESHOTBEHAVIOUR_H
#define EMBER_BEHAVIOUR_ONESHOTBEHAVIOUR_H

#include "Behaviour.h"

#include "Context.h"

#include "../Object.h"
#include "../Unit.h"

namespace ember {

namespace behaviour {

template<class AgentState=Unit, class Event=Unit>
class OneShotBehaviour:
    public Behaviour<AgentState, Event>,
    public Object<__ffi::OneShotBehaviour<__ffi::Event>> {
  public:
    OneShotBehaviour();
    virtual ~OneShotBehaviour();
    
    virtual void action(Context<Event>& context, AgentState& agent_state) const = 0;

  public:
    virtual void __ffi_add_behaviour_to_agent(__ffi::Agent<__ffi::AgentState, __ffi::Event>* agent) override;
    virtual void __ffi_add_behaviour_to_behaviour_vec(__ffi::BehaviourVec<__ffi::Event>* vec) override;
};

template<class AgentState, class E>
class OneShotBehaviour<AgentState, FsmEvent<E>>:
    public Behaviour<AgentState, FsmEvent<E>>,
    public Object<__ffi::OneShotBehaviour<__ffi::FsmEvent<const char*, __ffi::Event>>> {
  public:
    OneShotBehaviour();
    virtual ~OneShotBehaviour();
    
    virtual void action(Context<FsmEvent<E>>& context, AgentState& agent_state) const = 0;

  public:
    virtual uint32_t __ffi_add_behaviour_to_fsm_builder(
        __ffi::FsmBuilder<__ffi::AgentState, const char*, __ffi::Event>* builder,
        bool is_final
    ) override;
};

// ======================= Impl =======================

template<class AgentState, class Event>
OneShotBehaviour<AgentState, Event>::OneShotBehaviour():
    Object(
        __ffi::behaviour_oneshot_new(
            this,
            [](void* self_, __ffi::Context<__ffi::Event>* context_, __ffi::AgentState* agent_state_) {
                OneShotBehaviour<AgentState, Event>* self = static_cast<OneShotBehaviour<AgentState, Event>*>(self_);
                Context<Event> context(context_);
                AgentState& agent_state = *static_cast<AgentState*>(agent_state_->inner);
                return self->action(context, agent_state);
            }
        ),
        __ffi::behaviour_oneshot_free
    ) {}

template<class AgentState, class Event>
OneShotBehaviour<AgentState, Event>::~OneShotBehaviour() {}

template<class AgentState, class Event>
void OneShotBehaviour<AgentState, Event>::__ffi_add_behaviour_to_agent(__ffi::Agent<__ffi::AgentState, __ffi::Event>* agent) {
    __ffi::agent_add_behaviour_oneshot(
        agent,
        this->move_object()
    );
}

template<class AgentState, class Event>
void OneShotBehaviour<AgentState, Event>::__ffi_add_behaviour_to_behaviour_vec(__ffi::BehaviourVec<__ffi::Event>* vec) {
    __ffi::behaviour_vec_add_behaviour_oneshot(
        vec,
        this->move_object()
    );
}

// ======================= FsmEvent spec Impl =======================

template<class AgentState, class E>
OneShotBehaviour<AgentState, FsmEvent<E>>::OneShotBehaviour():
    Object(
        __ffi::behaviour_fsm_child_behaviour_oneshot_new(
            this,
            [](void* self_, __ffi::Context<__ffi::FsmEvent<const char*, __ffi::Event>>* context_, __ffi::AgentState* agent_state_) {
                OneShotBehaviour<AgentState, FsmEvent<E>>* self = static_cast<OneShotBehaviour<AgentState, FsmEvent<E>>*>(self_);
                Context<FsmEvent<E>> context(context_);
                AgentState& agent_state = *static_cast<AgentState*>(agent_state_->inner);
                return self->action(context, agent_state);
            }
        ),
        __ffi::behaviour_fsm_child_behaviour_oneshot_free
    ) {}

template<class AgentState, class E>
OneShotBehaviour<AgentState, FsmEvent<E>>::~OneShotBehaviour() {}

template<class AgentState, class E>
uint32_t OneShotBehaviour<AgentState, FsmEvent<E>>::__ffi_add_behaviour_to_fsm_builder(
    __ffi::FsmBuilder<__ffi::AgentState, const char*, __ffi::Event>* builder,
    bool is_final
) {
    return __ffi::behaviour_fsm_builder_add_behaviour_oneshot(builder, this->move_object(), is_final);
}

} // namespace behaviour

} // namespace ember

#endif // EMBER_BEHAVIOUR_ONESHOTBEHAVIOUR_H
