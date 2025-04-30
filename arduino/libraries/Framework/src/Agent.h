#ifndef FRAMEWORK_AGENT_H
#define FRAMEWORK_AGENT_H

#include <memory>

#include "FrameworkCore.h"

#include "Object.h"

#include "behaviour/Behaviour.h"

namespace framework {

template<typename Event=void>
class Agent: 
    public Object<__ffi::Agent<__ffi::Event>> {
  public:
    Agent(const char* const name);
    virtual ~Agent();
    
    void add_behaviour(std::unique_ptr<behaviour::Behaviour<Event>>&& behaviour);
};

// ======================= Impl =======================

template<typename Event>
Agent<Event>::Agent(const char* const name):
    Object(__ffi::agent_new(name), __ffi::agent_free) {}

template<typename Event>
Agent<Event>::~Agent() {};

template<typename Event>
void Agent<Event>::add_behaviour(std::unique_ptr<behaviour::Behaviour<Event>>&& behaviour_) {
    behaviour::Behaviour<Event>* behaviour = behaviour_.release();
    behaviour->__ffi_add_behaviour_to_agent(this->object);
}

} // namespace framework

#endif // FRAMEWORK_AGENT_H
