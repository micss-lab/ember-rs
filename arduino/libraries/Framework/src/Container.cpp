#include "Container.h"

#include <utility>

using namespace framework;

Container::Container(): 
    Object(__ffi::container_new(), __ffi::container_free) {}

// template<class Event>
// void Container::add_agent(Agent<Event>&& agent) {}

bool Container::start(Container&& container) {
    return static_cast<bool>(__ffi::container_start(container.move_object()));
}

Container::PollResult Container::poll() {
    return __ffi::container_poll(this->object);
}
