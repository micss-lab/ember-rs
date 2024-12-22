#include "Agent.h"

using namespace framework;

Agent::Agent(const char* const name): 
    Object(__ffi::agent_new(name), __ffi::agent_free) {}

Agent::~Agent() {};

void Agent::add_behaviour(std::unique_ptr<behaviour::Behaviour> behaviour) {
    behaviour->__ffi_add_behaviour_to_agent(this->value);
}

void Agent::free_object(__ffi::Agent<__ffi::Message>* agent) {
    __ffi::agent_free(agent);
}
