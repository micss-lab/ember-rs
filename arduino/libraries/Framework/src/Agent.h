#ifndef FRAMEWORK_AGENT_H
#define FRAMEWORK_AGENT_H

#include <memory>

#include "Behaviour.h"
#include "Object.h"
#include "FrameworkCore.h"

namespace framework {

class Agent: 
    public Object<__ffi::Agent> {
  public:
    Agent(const char* const name);
    virtual ~Agent();
    
    void add_behaviour(std::unique_ptr<behaviour::Behaviour> behaviour);

  protected:
    virtual void free_object(__ffi::Agent* agent);
};

} // namespace framework

#endif // FRAMEWORK_AGENT_H
