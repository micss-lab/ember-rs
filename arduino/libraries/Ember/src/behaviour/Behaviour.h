#ifndef EMBER_BEHAVIOUR_BEHAVIOUR_H
#define EMBER_BEHAVIOUR_BEHAVIOUR_H

#include <functional>
#include <memory>

#include "../EmberCore.h"

#include "../Object.h"
#include "../Unit.h"

namespace ember {

namespace behaviour {

// template: used to store only behaviours who pass around the same event and handle the same agent state.
template<class AgentState=Unit, class Event=void> 
class Behaviour {
  public:
    virtual void __ffi_add_behaviour_to_agent(__ffi::Agent<__ffi::AgentState, __ffi::Event>* agent) = 0;
    virtual void __ffi_add_behaviour_to_behaviour_vec(__ffi::BehaviourVec<__ffi::Event>* vec) = 0;
};

template<class AgentState=Unit, class Event=void> 
class BehaviourVec:
    public Object<__ffi::BehaviourVec<__ffi::Event>> {
  public:
    BehaviourVec();

    void add_behaviour(std::unique_ptr<Behaviour<AgentState, Event>>&& behaviour);
};

// ======================= Impl =======================

template<class AgentState, class Event>
BehaviourVec<AgentState, Event>::BehaviourVec():
    Object(__ffi::behaviour_vec_new(), __ffi::behaviour_vec_free) {}

template<class AgentState, class Event>
void BehaviourVec<AgentState, Event>::add_behaviour(std::unique_ptr<Behaviour<AgentState, Event>>&& behaviour_) {
    Behaviour<AgentState, Event>* behaviour = behaviour_.release();
    behaviour->__ffi_add_behaviour_to_behaviour_vec(this->object);
}

} // namespace behaviour

} // namespace ember

#endif // EMBER_BEHAVIOUR_BEHAVIOUR_H
