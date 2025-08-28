#ifndef EMBER_BEHAVIOUR_SEQUENTIALBEHAVIOUR_H
#define EMBER_BEHAVIOUR_SEQUENTIALBEHAVIOUR_H

#include <vector>
#include <memory>

#include "Behaviour.h"

#include "Context.h"

#include "../Object.h"
#include "../Unit.h"

namespace ember {

namespace behaviour {

template<class AgentState=Unit, class E=void, class ChildEvent=void>
class SequentialBehaviour:
    public Behaviour<AgentState, E>,
    public Object<__ffi::SequentialBehaviour> {
  public:
    SequentialBehaviour(BehaviourVec<AgentState, ChildEvent>&& initial_behaviours);
    virtual ~SequentialBehaviour();

    virtual void handle_child_event(Event<ChildEvent>&& event);

    virtual void after_child_action(Context<E>& context, AgentState& agent_state);

  public:
    virtual void __ffi_add_behaviour_to_agent(__ffi::Agent<__ffi::AgentState, __ffi::Event>* agent) override;
    virtual void __ffi_add_behaviour_to_behaviour_vec(__ffi::BehaviourVec* vec) override;
};

// ======================= Impl =======================

template<class AgentState, class E, class ChildEvent>
SequentialBehaviour<AgentState, E, ChildEvent>::SequentialBehaviour(BehaviourVec<AgentState, ChildEvent>&& initial_behaviours):
    Object(
        __ffi::behaviour_sequential_new(
            this,
            initial_behaviours.move_object(),
            [](void* self_, __ffi::Event* event_) {
                SequentialBehaviour<AgentState, E, ChildEvent>* self = static_cast<SequentialBehaviour<AgentState, E, ChildEvent>*>(self_);
                Event<ChildEvent> event(event_);
                self->handle_child_event(std::move(event));
            },
            [](void* self_, __ffi::Context<__ffi::Event>* context_, __ffi::AgentState* agent_state_) {
                SequentialBehaviour<AgentState, E, ChildEvent>* self = static_cast<SequentialBehaviour<AgentState, E, ChildEvent>*>(self_);
                Context<E> context(context_);
                AgentState& agent_state = *static_cast<AgentState*>(agent_state_->inner);
                self->after_child_action(context, agent_state);
            }
        ),
        __ffi::behaviour_sequential_free
    ) {}

template<class AgentState, class E, class ChildEvent>
SequentialBehaviour<AgentState, E, ChildEvent>::~SequentialBehaviour() {}

template<class AgentState, class E, class ChildEvent>
void SequentialBehaviour<AgentState, E, ChildEvent>::handle_child_event(Event<ChildEvent>&&) {
    // Does nothing.
}

template<class AgentState, class E, class ChildEvent>
void SequentialBehaviour<AgentState, E, ChildEvent>::after_child_action(Context<E>& context, AgentState& agent_state) {
    // Does nothing.
}

template<class AgentState, class E, class ChildEvent>
void SequentialBehaviour<AgentState, E, ChildEvent>::__ffi_add_behaviour_to_agent(
    __ffi::Agent<__ffi::AgentState, __ffi::Event>* agent
) {
    __ffi::agent_add_behaviour_sequential(
        agent,
        this->move_object()
    );
}

template<class AgentState, class E, class ChildEvent>
void SequentialBehaviour<AgentState, E, ChildEvent>::__ffi_add_behaviour_to_behaviour_vec(__ffi::BehaviourVec* vec) {
    __ffi::behaviour_vec_add_behaviour_sequential(
        vec,
        this->move_object()
    );
}

} // namespace behaviour

} // namespace ember

#endif // EMBER_BEHAVIOUR_SEQUENTIALBEHAVIOUR_H
