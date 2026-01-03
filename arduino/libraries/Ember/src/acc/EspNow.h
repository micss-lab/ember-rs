#ifndef EMBER_ACC_ESPNOW_H
#define EMBER_ACC_ESPNOW_H

// #ifdef EMBER_ENABLE_ACC_ESPNOW

#include <array>
#include <memory>
#include <unordered_map>

#include "ESP32_NOW.h"

#include "Acc.h"

#include "../EmberCore.h"

namespace ember {

namespace acc {

class EspNowAcc;

class EspNowPeer:
    public ESP_NOW_Peer {
  private:
    using MessageEnvelope = ember::message::MessageEnvelope;

  public:
    EspNowPeer(const uint8_t *mac_addr, uint8_t channel, wifi_interface_t iface, const uint8_t *lmk);
    ~EspNowPeer();

  public:
    void set_channel(std::weak_ptr<EspNowAcc> channel);

    void send_message(const MessageEnvelope& envelope);

  public:
    /**
     * Overrides the receive call to delegate to the channel.
     *
     * IMPORTANT: This should never be overridden by the user.
     */
    void onReceive(const uint8_t* data, size_t len, bool _broadcast) override final;

  private:
    std::weak_ptr<EspNowAcc> channel;
};

class EspNowAcc:
    public Acc,
    public std::enable_shared_from_this<EspNowAcc> {
  public:
    EspNowAcc() = default;

    // Delete anything that allows this object to move in memory once created.
    // The peer heavily depends on the pointer to this class remaining the same.
    // This forces the user to use this class behind a pointer.
    EspNowAcc(EspNowAcc&&) = delete;
    EspNowAcc& operator=(EspNowAcc&&) = delete;
    EspNowAcc(const EspNowAcc&) = delete;
    EspNowAcc& operator=(const EspNowAcc&) = delete;

    void add_peer(std::unique_ptr<EspNowPeer>&& peer);

  public:
    void handle_on_receive(const uint8_t* data, size_t len);

  public:
    bool send(const char* aid, const MessageEnvelope& envelope) override;
    std::optional<MessageEnvelope> receive() override;

  private:
    std::unordered_map<uint64_t, std::unique_ptr<EspNowPeer>> peers;
};

} // namespace acc

} // namespace ember

// #endif // EMBER_ENABLE_ACC_ESPNOW

#endif // EMBER_ACC_ESPNOW_H