#include <string>
#include <cstring>

#include "Arduino_DebugUtils.h"
#include "WiFi.h"
#include "ESP32_NOW.h"

#include "Ember.h"

#define ESPNOW_WIFI_CHANNEL 6

struct Message {
    std::string message;

    static const char* const decode_message(const ember::message::Message& message_) {
        ember::message::ContentView content = message_.get_content();

        return reinterpret_cast<const char* const>(content.data);
    }
};

class AgentB:
    public ember::behaviour::TickerBehaviour<> {
  public:
    void action(ember::behaviour::Context<>& context, ember::Unit& agent_state) override {
        auto message = context.receive_message_with_filter(ember::message::MessageFilter::ontology("some_ontology"));
        if (!message.has_value()) {
          context.block_behaviour();
          return;
        }
        Serial.println("Ow, I got a message...");
        Serial.print("It reads: ");
        Serial.println(Message::decode_message(message.value()));
    }

    uint64_t interval_millis() const override {
        return 2000;
    }

    bool is_finished() const override {
        return false;
    }
};

ember::Agent<> create_receiver_agent() {
    ember::Agent<> receiver_agent{"agent-b", std::move(ember::Unit{})};
    auto sender_behaviour = std::make_unique<AgentB>();
    receiver_agent.add_behaviour(std::move(sender_behaviour));
    return std::move(receiver_agent);
}

std::unique_ptr<ember::Container> container;
std::shared_ptr<ember::acc::EspNowAcc> espnow;

// Callback called when an unknown peer sends a message
void register_sender(const esp_now_recv_info_t *info, const uint8_t *data, int len, void *arg) {
  auto sender = std::make_unique<ember::acc::EspNowPeer>(info->src_addr, ESPNOW_WIFI_CHANNEL, WIFI_IF_STA, nullptr);
  espnow->add_peer(std::move(sender));
  Serial.printf("Registering new sender with address " MACSTR, MAC2STR(info->src_addr));
}


void setup() {
  Debug.setDebugLevel(DBG_VERBOSE);

  Serial.begin(115200);

  // Initialize the Wi-Fi module
  WiFi.mode(WIFI_STA);
  WiFi.setChannel(ESPNOW_WIFI_CHANNEL);
  while (!WiFi.STA.started()) {
    delay(100);
  }

  // Initialize the embers required resources.
  ember::initialize(ember::logging::LogLevel::Debug);

  // Create the main container instance.
  container = std::make_unique<ember::Container>();

  // Create the espnow communication channel.
  espnow = container->enable_acc_espnow();
  if (!ESP_NOW.begin()) {
    throw "Failed to start espnow channel";
  }
  ESP_NOW.onNewPeer(register_sender, nullptr);

  auto receiver = create_receiver_agent();

  container->add_agent(std::move(receiver));
}

void loop() {
  ember::Container::PollResult result = container->poll();
  if (result.should_stop) {
    Serial.println(
      (result.status == 0) ? "Finished executing." : "Container exited with an error!"
    );
    exit(result.status);
  }
}