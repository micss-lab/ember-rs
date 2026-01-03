#include <string>
#include <cstring>

#include "Arduino_DebugUtils.h"
#include "WiFi.h"

#include "Ember.h"

#define ESPNOW_WIFI_CHANNEL 6

struct Message {
    std::string message;

    ember::message::Message into_message() const {
        // Serialize: copy string content into vector
        std::vector<uint8_t> content(message.begin(), message.end());
        content.push_back('\0');

        return ember::message::Message(
            ember::message::Performative::Inform,
            // The agents mac address will be resolved using the proxy.
            {"agent-b@local"},
            "some_ontology",
            content
        );
    }
};

class AgentA:
    public ember::behaviour::TickerBehaviour<> {
  public:
    void action(ember::behaviour::Context<>& context, ember::Unit& agent_state) override {
        ember::message::MessageEnvelope message = Message{
          .message = "I have something to tell you..."
        }.into_message().wrap_with_envelope();
        context.send_message(std::move(message));
    }

    uint64_t interval_millis() const override {
        return 5000;
    }

    bool is_finished() const override {
        return false;
    }
};

ember::Agent<> create_sender_agent() {
    ember::Agent<> sender_agent{"agent-a", std::move(ember::Unit{})};
    auto sender_behaviour = std::make_unique<AgentA>();
    sender_agent.add_behaviour(std::move(sender_behaviour));
    return std::move(sender_agent);
}

std::unique_ptr<ember::Container> container;

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
  auto espnow = container->enable_acc_espnow();
  auto agent_b = std::make_unique<ember::acc::EspNowPeer>(ESP_NOW.BROADCAST_ADDR, ESPNOW_WIFI_CHANNEL, WIFI_IF_STA, nullptr);
  espnow->add_peer(std::move(agent_b));

  auto sender = create_sender_agent();

  container->add_agent(std::move(sender));
  container->add_agent_proxy("agent-b", "agent-b@ff:ff:ff:ff:ff:ff");
}

void loop() {
  try {
    ember::Container::PollResult result = container->poll();
    if (result.should_stop) {
      Serial.println(
        (result.status == 0) ? "Finished executing." : "Container exited with an error!"
      );
      exit(result.status);
    }
  } catch (const char* e) {
    Serial.println(e);
  } catch (std::exception e) {
    Serial.println(e.what());
  }
}