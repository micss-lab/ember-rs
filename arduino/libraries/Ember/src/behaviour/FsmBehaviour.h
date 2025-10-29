#ifndef EMBER_BEHAVIOUR_FSMBEHAVIOUR_H
#define EMBER_BEHAVIOUR_FSMBEHAVIOUR_H

#include <vector>
#include <memory>

#include "Behaviour.h"

#include "Context.h"
#include "Event.h"

#include "../Object.h"
#include "../Unit.h"

namespace ember {

namespace behaviour {

template<class AgentState, class E>
class FsmBuilder;

template<class AgentState=Unit, class E=Unit>
class Fsm: 
    public Object<__ffi::Fsm<__ffi::AgentState, const char*, __ffi::Event>> {
  public:
    Fsm(__ffi::Fsm<__ffi::AgentState, const char*, __ffi::Event>* fsm);
    virtual ~Fsm();

    static FsmBuilder<AgentState, E> builder() {
        return std::move(FsmBuilder<AgentState, E>());
    }
};

template<class AgentState=Unit, class E=Unit>
class FsmBuilder:
    public Object<__ffi::FsmBuilder<__ffi::AgentState, const char*, __ffi::Event>> {
  public:
    FsmBuilder();
    virtual ~FsmBuilder();

    uint32_t add_behaviour(std::unique_ptr<Behaviour<AgentState, FsmEvent<E>>>&& behaviour, bool is_final);
    void add_transition(uint32_t src, uint32_t dest, std::optional<const char*> trigger);

    Fsm<AgentState, E> build(uint32_t start_behaviour);
};

template<class AgentState=Unit, class E=Unit, class ChildEvent=Unit>
class FsmBehaviour:
    public Behaviour<AgentState, E>,
    public Object<__ffi::FsmBehaviour<__ffi::Event>> {
  public:
    FsmBehaviour(Fsm<AgentState, ChildEvent>&& fsm);
    virtual ~FsmBehaviour();

    virtual void handle_child_event(Event<ChildEvent>&& event);

    virtual void after_child_action(Context<E>& context, AgentState& agent_state);

  public:
    virtual void __ffi_add_behaviour_to_agent(__ffi::Agent<__ffi::AgentState, __ffi::Event>* agent) override;
    virtual void __ffi_add_behaviour_to_behaviour_vec(__ffi::BehaviourVec<__ffi::Event>* vec) override;
};

template<class AgentState, class E, class ChildEvent>
class FsmBehaviour<AgentState, FsmEvent<E>, ChildEvent>:
    public Behaviour<AgentState, FsmEvent<E>>,
    public Object<__ffi::FsmBehaviour<__ffi::FsmEvent<const char*, __ffi::Event>>> {
  public:
    FsmBehaviour(Fsm<AgentState, ChildEvent>&& fsm);
    virtual ~FsmBehaviour();

    virtual void handle_child_event(Event<ChildEvent>&& event);

    virtual void after_child_action(Context<FsmEvent<E>>& context, AgentState& agent_state);

  public:
    virtual uint32_t __ffi_add_behaviour_to_fsm_builder(
        __ffi::FsmBuilder<__ffi::AgentState, const char*, __ffi::Event>* builder,
        bool is_final
    ) override;
};

// ======================= Fsm Impl =======================

template<class AgentState, class E>
Fsm<AgentState, E>::Fsm(__ffi::Fsm<__ffi::AgentState, const char*, __ffi::Event>* fsm):
    Object(fsm, nullptr) {}

template<class AgentState, class E>
Fsm<AgentState, E>::~Fsm() {}

// ======================= FsmBuilder Impl =======================

template<class AgentState, class E>
FsmBuilder<AgentState, E>::FsmBuilder():
    Object(
        __ffi::behaviour_fsm_builder_new(),
        nullptr  // FsmBuilder ownership is transferred in build()
    ) {}

template<class AgentState, class E>
FsmBuilder<AgentState, E>::~FsmBuilder() {}

template<class AgentState, class E>
uint32_t FsmBuilder<AgentState, E>::add_behaviour(
    std::unique_ptr<Behaviour<AgentState, FsmEvent<E>>>&& behaviour,
    bool is_final
) {
    return behaviour->__ffi_add_behaviour_to_fsm_builder(this->object, is_final);
}

template<class AgentState, class E>
void FsmBuilder<AgentState, E>::add_transition(
    uint32_t src,
    uint32_t dest,
    std::optional<const char*> trigger
) {
    if (trigger.has_value()) {
        __ffi::behaviour_fsm_builder_add_transition(this->object, src, dest, trigger.value());
    } else {
        __ffi::behaviour_fsm_builder_add_default_transition(this->object, src, dest);
    }
}

template<class AgentState, class E>
Fsm<AgentState, E> FsmBuilder<AgentState, E>::build(__ffi::BehaviourId start_behaviour) {
    __ffi::Fsm<__ffi::AgentState, const char*, __ffi::Event>* fsm = __ffi::behaviour_fsm_builder_build(
        this->move_object(),
        start_behaviour
    );
    return Fsm<AgentState, E>(fsm);
}

// ======================= FsmBehaviour Impl =======================

template<class AgentState, class E, class ChildEvent>
FsmBehaviour<AgentState, E, ChildEvent>::FsmBehaviour(Fsm<AgentState, ChildEvent>&& fsm):
    Object(
        __ffi::behaviour_fsm_behaviour_new(
            this,
            fsm.move_object(),
            [](void* self_, __ffi::Event* event_) {
                FsmBehaviour<AgentState, E, ChildEvent>* self = static_cast<FsmBehaviour<AgentState, E, ChildEvent>*>(self_);
                Event<ChildEvent> event(event_);
                self->handle_child_event(std::move(event));
            },
            [](void* self_, __ffi::Context<__ffi::Event>* context_, __ffi::AgentState* agent_state_) {
                FsmBehaviour<AgentState, E, ChildEvent>* self = static_cast<FsmBehaviour<AgentState, E, ChildEvent>*>(self_);
                Context<E> context(context_);
                AgentState& agent_state = *static_cast<AgentState*>(agent_state_->inner);
                self->after_child_action(context, agent_state);
            }
        ),
        __ffi::behaviour_fsm_behaviour_free
    ) {}

template<class AgentState, class E, class ChildEvent>
FsmBehaviour<AgentState, E, ChildEvent>::~FsmBehaviour() {}

template<class AgentState, class E, class ChildEvent>
void FsmBehaviour<AgentState, E, ChildEvent>::handle_child_event(Event<ChildEvent>&&) {
    // Does nothing.
}

template<class AgentState, class E, class ChildEvent>
void FsmBehaviour<AgentState, E, ChildEvent>::after_child_action(Context<E>& context, AgentState& agent_state) {
    // Does nothing.
}

template<class AgentState, class E, class ChildEvent>
void FsmBehaviour<AgentState, E, ChildEvent>::__ffi_add_behaviour_to_agent(
    __ffi::Agent<__ffi::AgentState, __ffi::Event>* agent
) {
    __ffi::agent_add_behaviour_fsm(
        agent,
        this->move_object()
    );
}

template<class AgentState, class E, class ChildEvent>
void FsmBehaviour<AgentState, E, ChildEvent>::__ffi_add_behaviour_to_behaviour_vec(__ffi::BehaviourVec<__ffi::Event>* vec) {
    __ffi::behaviour_vec_add_behaviour_fsm(
        vec,
        this->move_object()
    );
}

// ======================= FsmEvent spec FsmBehaviour Impl =======================

template<class AgentState, class E, class ChildEvent>
FsmBehaviour<AgentState, FsmEvent<E>, ChildEvent>::FsmBehaviour(Fsm<AgentState, ChildEvent>&& fsm):
    Object(
        __ffi::behaviour_fsm_child_behaviour_fsm_new(
            this,
            fsm.move_object(),
            [](void* self_, __ffi::Event* event_) {
                FsmBehaviour<AgentState, FsmEvent<E>, ChildEvent>* self = static_cast<FsmBehaviour<AgentState, FsmEvent<E>, ChildEvent>*>(self_);
                Event<ChildEvent> event(event_);
                self->handle_child_event(std::move(event));
            },
            [](void* self_, __ffi::Context<__ffi::FsmEvent<const char*, __ffi::Event>>* context_, __ffi::AgentState* agent_state_) {
                FsmBehaviour<AgentState, FsmEvent<E>, ChildEvent>* self = static_cast<FsmBehaviour<AgentState, FsmEvent<E>, ChildEvent>*>(self_);
                Context<FsmEvent<E>> context(context_);
                AgentState& agent_state = *static_cast<AgentState*>(agent_state_->inner);
                self->after_child_action(context, agent_state);
            }
        ),
        __ffi::behaviour_fsm_child_behaviour_fsm_free
    ) {}

template<class AgentState, class E, class ChildEvent>
FsmBehaviour<AgentState, FsmEvent<E>, ChildEvent>::~FsmBehaviour() {}

template<class AgentState, class E, class ChildEvent>
void FsmBehaviour<AgentState, FsmEvent<E>, ChildEvent>::handle_child_event(Event<ChildEvent>&&) {
    // Does nothing.
}

template<class AgentState, class E, class ChildEvent>
void FsmBehaviour<AgentState, FsmEvent<E>, ChildEvent>::after_child_action(Context<FsmEvent<E>>& context, AgentState& agent_state) {
    // Does nothing.
}

template<class AgentState, class E, class ChildEvent>
uint32_t FsmBehaviour<AgentState, FsmEvent<E>, ChildEvent>::__ffi_add_behaviour_to_fsm_builder(
    __ffi::FsmBuilder<__ffi::AgentState, const char*, __ffi::Event>* builder,
    bool is_final
) {
    return __ffi::behaviour_fsm_builder_add_behaviour_fsm(builder, this->move_object(), is_final);
}

} // namespace behaviour

} // namespace ember

#endif // EMBER_BEHAVIOUR_FSMBEHAVIOUR_H