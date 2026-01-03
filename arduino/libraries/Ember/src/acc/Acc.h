#ifndef EMBER_ACC_ACC_H
#define EMBER_ACC_ACC_H

#define EMBER_ACC_MESSAGE_QUEUE_SIZE 10

#include <optional>
#include <array>

#include "Arduino_DebugUtils.h"

#include "../message/Message.h"

namespace ember {

namespace acc {

template<std::size_t N>
class MessageQueue {
  private:
    using MessageEnvelope = ember::message::MessageEnvelope;

  public:
    void append(MessageEnvelope&& message);
    std::optional<MessageEnvelope> pop();

  private:
    size_t current{0};
    std::array<std::optional<MessageEnvelope>, N> queue;
};

class Acc {
  protected:
    using MessageEnvelope = ember::message::MessageEnvelope;

  public:
    virtual ~Acc() = default;
    virtual bool send(const char* aid, const MessageEnvelope& envelope) = 0;
    virtual std::optional<MessageEnvelope> receive() = 0;

  protected:
    MessageQueue<EMBER_ACC_MESSAGE_QUEUE_SIZE> queue;
};

// ======================= Impl =======================

template<std::size_t N>
void MessageQueue<N>::append(MessageEnvelope&& message) {
    DEBUG_VERBOSE("Appending message to queue");
    DEBUG_DEBUG("Current idx = %i", this->current);
    if (this->queue[this->current].has_value()) {
      DEBUG_WARNING("Dropping message from queue as the ring buffer is full.");
    }
    this->queue[this->current] = std::optional(std::move(message));
    this->current = (this->current + 1) % N;
}

template<std::size_t N>
std::optional<ember::message::MessageEnvelope> MessageQueue<N>::pop() {
    for (size_t offset = 0; offset < N; offset++) {
        size_t idx = (this->current + offset) % N;
        if (this->queue[idx].has_value()) {
            DEBUG_DEBUG("Current idx = %i, idx = %i", this->current, idx);
            DEBUG_VERBOSE("Returning message from queue.");
            std::optional<MessageEnvelope> result = std::move(this->queue[idx]);
            this->queue[idx] = std::nullopt;
            return result;
        }
    }
    return std::nullopt;
}

}; // namespace acc

}; // namespace ember

#endif // EMBER_ACC_ACC_H