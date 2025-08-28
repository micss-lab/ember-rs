#ifndef EMBER_CONTAINER_H
#define EMBER_CONTAINER_H

#include "EmberCore.h"

#include "Object.h"

#include "Agent.h"

namespace ember {

class Container: 
    public Object<__ffi::Container> {
  public:
    using PollResult = __ffi::ContainerPollResult;

  public:
    Container();

    template<class Event>
    void add_agent(Agent<Event>&& agent);

  public:
    PollResult poll();
  
  public:
    static bool start(Container&& container);
};

// ======================= Impl =======================

template<class Event>
void Container::add_agent(Agent<Event>&& agent) {
    __ffi::container_add_agent(this->object, agent.move_object());
}

}

#endif // EMBER_CONTAINER_H
