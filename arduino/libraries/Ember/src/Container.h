#ifndef EMBER_CONTAINER_H
#define EMBER_CONTAINER_H

#include "EmberCore.h"

#include "Object.h"

#include "Agent.h"

// #ifdef EMBER_ENABLE_ACC_ESPNOW
#include "./acc/EspNow.h"
// #endif // EMBER_ENABLE_ACC_ESPNOW

namespace ember {

class Container:
    public Object<__ffi::Container> {
  public:
    using PollResult = __ffi::ContainerPollResult;

  public:
    Container();

    template<class Event>
    void add_agent(Agent<Event>&& agent);
    void add_agent_proxy(const char* local_name, const char* aid);

    // #ifdef EMBER_ENABLE_ACC_ESPNOW
    std::shared_ptr<acc::EspNowAcc> enable_acc_espnow();
    // #endif // EMBER_ENABLE_ACC_ESPNOW

  public:
    PollResult poll();

  public:
    static bool start(Container&& container);

  private:
    // #ifdef EMBER_ENABLE_ACC_ESPNOW
    std::optional<std::shared_ptr<acc::EspNowAcc>> espnow_acc;
    // #endif // EMBER_ENABLE_ACC_ESPNOW
};

// ======================= Impl =======================

template<class Event>
void Container::add_agent(Agent<Event>&& agent) {
    __ffi::container_add_agent(this->object, agent.move_object());
}

}

#endif // EMBER_CONTAINER_H
