#ifndef FRAMEWORK_AGENT_H
#define FRAMEWORK_AGENT_H

#include <memory>

#include "FrameworkCore.h"

#include "Object.h"

#include "behaviour/Behaviour.h"

namespace framework {

template<typename Message=void>
class Agent: 
    public Object<__ffi::Agent<__ffi::Message>> {
  public:
    Agent(const char* const name);
    virtual ~Agent();
    
    void add_behaviour(std::unique_ptr<behaviour::Behaviour<Message>>&& behaviour);
};

// ======================= Impl =======================

template<typename Message>
Agent<Message>::Agent(const char* const name):
    Object(__ffi::agent_new(name), __ffi::agent_free) {}

template<typename Message>
Agent<Message>::~Agent() {};

template<typename Message>
void Agent<Message>::add_behaviour(std::unique_ptr<behaviour::Behaviour<Message>>&& behaviour_) {
    behaviour::Behaviour<Message>* behaviour = behaviour_.release();
    behaviour->__ffi_add_behaviour_to_agent(this->object);
}

} // namespace framework

#endif // FRAMEWORK_AGENT_H
