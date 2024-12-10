#include "Container.h"

#include <utility>

using namespace framework;

Container::Container(): 
    Object(__ffi::container_new(), __ffi::container_free) {}

void Container::add_agent(Agent&& agent) {
    __ffi::container_add_agent(this->value, agent.move_object());
}

bool Container::start(Container&& container) {
    return static_cast<bool>(__ffi::container_start(container.move_object()));
}
