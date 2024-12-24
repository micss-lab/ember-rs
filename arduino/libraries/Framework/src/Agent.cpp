#include "Agent.h"

using namespace framework;

Agent::Agent(const char* const name): 
    Object(__ffi::agent_new(name), __ffi::agent_free) {}

Agent::~Agent() {};

void Agent::add_behaviour(std::unique_ptr<behaviour::Behaviour>&& behaviour_) {
    behaviour::Behaviour* behaviour = behaviour_.release();
    behaviour->__ffi_add_behaviour_to_agent(this->object);
}
