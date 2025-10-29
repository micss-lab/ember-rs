#ifndef EMBER_AGENT_H
#define EMBER_AGENT_H

#include <memory>

#include "EmberCore.h"

#include "Object.h"
#include "Unit.h"

#include "behaviour/Behaviour.h"

namespace ember {

template<typename State=Unit, typename Event=Unit>
class Agent: 
    public Object<__ffi::Agent<__ffi::AgentState, __ffi::Event>> {
  public:
    Agent(const char* const name, State&& state);
    Agent(Agent&&) = default;
    virtual ~Agent();
    
    void add_behaviour(std::unique_ptr<behaviour::Behaviour<State, Event>>&& behaviour);
};

// ======================= Impl =======================

template<typename State, typename Event>
Agent<State, Event>::Agent(const char* const name, State&& state):
    Object(__ffi::agent_new(name, __ffi::agent_state_new(new State(state))), __ffi::agent_free) {}

template<typename State, typename Event>
Agent<State, Event>::~Agent() {};

template<typename State, typename Event>
void Agent<State, Event>::add_behaviour(std::unique_ptr<behaviour::Behaviour<State, Event>>&& behaviour_) {
    behaviour::Behaviour<State, Event>* behaviour = behaviour_.release();
    behaviour->__ffi_add_behaviour_to_agent(this->object);
}

} // namespace ember

#endif // EMBER_AGENT_H
