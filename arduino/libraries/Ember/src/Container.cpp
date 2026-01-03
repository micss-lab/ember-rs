#include "Container.h"

#include <utility>

using namespace ember;

Container::Container():
    Object(__ffi::container_new(), __ffi::container_free) {}

// template<class Event>
// void Container::add_agent(Agent<Event>&& agent) {}

void Container::add_agent_proxy(const char* local_name, const char* aid) {
    __ffi::container_add_agent_proxy(this->object, local_name, aid);
}

// #ifdef EMBER_ENABLE_ACC_ESPNOW
std::shared_ptr<acc::EspNowAcc> Container::enable_acc_espnow() {
    this->espnow_acc = std::optional(std::make_shared<acc::EspNowAcc>());
    const auto acc = __ffi::acc_custom_acc_new(
        this->espnow_acc.value().get(),
        [](void* espnow_acc_, const char* aid, __ffi::MessageEnvelope* envelope_) {
            acc::EspNowAcc* espnow_acc = static_cast<acc::EspNowAcc*>(espnow_acc_);
            message::MessageEnvelope envelope(envelope_);
            return espnow_acc->send(aid, envelope);
        },
        [](void* espnow_acc_) {
            acc::EspNowAcc* espnow_acc = static_cast<acc::EspNowAcc*>(espnow_acc_);
            auto result = espnow_acc->receive();
            if (!result.has_value()) {
                return static_cast<__ffi::MessageEnvelope*>(nullptr);
            }
            return result.value().move_object();
        }
    );
    __ffi::container_enable_custom_acc(this->object, acc);
    return this->espnow_acc.value();
}
// #endif // EMBER_ENABLE_ACC_ESPNOW

bool Container::start(Container&& container) {
    return static_cast<bool>(__ffi::container_start(container.move_object()));
}

Container::PollResult Container::poll() {
    return __ffi::container_poll(this->object);
}
