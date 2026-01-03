#include "EspNow.h"

#include <cstdint>
#include <cstring>
#include <cctype>
#include <exception>

#include "Arduino_DebugUtils.h"

using namespace ember::acc;

uint64_t ptr_to_mac(const uint8_t* ptr) {
    uint64_t mac = 0;
    for (int i = 0; i < 6; ++i) {
        mac |= ((uint64_t)ptr[i]) << (i * 8);
    }
    return mac;
}

uint64_t aid_to_mac(const char* aid) {
    const char* mac = strchr(aid, '@');
    if (!mac || strlen(++mac) != 17) throw "Invalid AID format";

    uint64_t result = 0;
    for (int i = 0; i < 6; i++) {
        if (i > 0 && mac[i*3-1] != ':') throw "Invalid MAC format";
        if (!isxdigit(mac[i*3]) || !isxdigit(mac[i*3+1])) throw "Invalid hex digit";

        auto hex = [](char c) {
            return c >= 'A' ? (c & 0xF) + 9 : c & 0xF;
        };
        result = (result << 8) | (hex(mac[i*3]) << 4) | hex(mac[i*3+1]);
    }
    return result;
}

EspNowPeer::EspNowPeer(
    const uint8_t *mac_addr,
    uint8_t channel,
    wifi_interface_t iface,
    const uint8_t *lmk
): ESP_NOW_Peer(mac_addr, channel, iface, lmk) {
    if (!ESP_NOW.begin()) {
        throw "Failed to start espnow network";
    }
    if (!this->add()) {
        throw "Failed to add peer to espnow network";
    }
}

EspNowPeer::~EspNowPeer() {
    this->remove();
}

void EspNowPeer::set_channel(std::weak_ptr<EspNowAcc> channel) {
    this->channel = channel;
}

void EspNowPeer::send_message(const MessageEnvelope& envelope) {
    DEBUG_VERBOSE("Sending message to espnow peer.");
    const __ffi::PostcardBytes data = envelope.__ffi_to_postcard_bytes();
    if (!this->send(data.data, data.len)) {
        throw "Error occured sending message";
    }
    __ffi::postcard_bytes_free(data);
}

void EspNowPeer::onReceive(const uint8_t* data, size_t len, bool _broadcast) {
    DEBUG_VERBOSE("Receiving message from peer.");
    if (this->channel.expired()) {
        throw "EspNow channel pointer is invalid.";
    }
    this->channel.lock()->handle_on_receive(data, len);
}

void EspNowAcc::add_peer(std::unique_ptr<EspNowPeer>&& peer) {
    // SAFETY: This class is not moveable in memory.
    peer->set_channel(this->shared_from_this());
    const uint64_t mac = ptr_to_mac(peer->addr());
    this->peers.insert_or_assign(mac, std::move(peer));
}

void EspNowAcc::handle_on_receive(const uint8_t* data, size_t len) {
    this->queue.append(MessageEnvelope(__ffi::message_envelope_deserialize_from_postcard_bytes(data, len)));
}

bool EspNowAcc::send(const char* aid, const MessageEnvelope& envelope) {
    uint64_t mac = aid_to_mac(aid);
    try {
        this->peers.at(mac)->send_message(envelope);
    } catch (std::out_of_range e) {
        // TODO: Log that the aid is invalid.
        return false;
    }
    return true;
}

std::optional<ember::message::MessageEnvelope> EspNowAcc::receive() {
    return std::move(this->queue.pop());
}