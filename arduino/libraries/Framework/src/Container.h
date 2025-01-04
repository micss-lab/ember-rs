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

    template<class Message>
    void add_agent(Agent<Message>&& agent);

  public:
    PollResult poll();
  
  public:
    static bool start(Container&& container);
};

// ======================= Impl =======================

template<class Message>
void Container::add_agent(Agent<Message>&& agent) {
    __ffi::container_add_agent(this->object, agent.move_object());
}

}

#endif // FRAMEWORK_CONTAINER_H
