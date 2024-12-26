#ifndef FRAMEWORK_CONTAINER_H
#define FRAMEWORK_CONTAINER_H

#include "FrameworkCore.h"

#include "Object.h"

#include "Agent.h"

namespace framework {

class Container: 
    public Object<__ffi::Container> {
  public:
    using PollResult = __ffi::ContainerPollResult;

  public:
    Container();

    void add_agent(Agent&& agent);

  public:
    PollResult poll();
  
  public:
    static bool start(Container&& container);
};

}

#endif // FRAMEWORK_CONTAINER_H
