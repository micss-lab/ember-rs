#ifndef FRAMEWORK_AGENT_H
#define FRAMEWORK_AGENT_H

#include <memory>

#include "FrameworkCore.h"

#include "Object.h"

#include "behaviour/Behaviour.h"

namespace framework {

class Agent: 
    public Object<__ffi::Agent<__ffi::Message>> {
  public:
    Agent(const char* const name);
    virtual ~Agent();
    
    void add_behaviour(std::unique_ptr<behaviour::Behaviour>&& behaviour);
};

} // namespace framework

#endif // FRAMEWORK_AGENT_H
