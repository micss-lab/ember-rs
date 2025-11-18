#include "Ember.h"

#include <string>
#include <cstring>

struct Message {
    std::string message;

    static const char* const decode_message(const ember::message::Message& message_) {
        ember::message::ContentView content = message_.get_content();
        
        return reinterpret_cast<const char* const>(content.data);
    }

    ember::message::Message into_message() const {
        // Serialize: copy string content into vector
        std::vector<uint8_t> content(message.begin(), message.end());
        content.push_back('\0');
        
        return ember::message::Message(
            ember::message::Performative::Inform, 
            {"receiver@local"}, 
            "some_ontology", 
            content
        );
    }
};

class Sender:
    public ember::behaviour::CyclicBehaviour<> {
  public:
    void action(ember::behaviour::Context<>& context, ember::Unit& agent_state) override {
        ember::message::MessageEnvelope message = Message{
          .message = "I have something to tell you..."
        }.into_message().wrap_with_envelope();
        context.send_message(std::move(message));
    }

    bool is_finished() const override {
        return false;
    }
};

class Receiver:
    public ember::behaviour::CyclicBehaviour<> {
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

    bool is_finished() const override {
        return false;
    }
};

ember::Agent<> create_sender_agent() {
    ember::Agent<> sender_agent{"sender", std::move(ember::Unit{})};
    auto sender_behaviour = std::make_unique<Sender>();
    sender_agent.add_behaviour(std::move(sender_behaviour));
    return std::move(sender_agent);
}

ember::Agent<> create_receiver_agent() {
    ember::Agent<> receiver_agent{"receiver", std::move(ember::Unit{})};
    auto receiver_behaviour = std::make_unique<Receiver>();
    receiver_agent.add_behaviour(std::move(receiver_behaviour));
    return std::move(receiver_agent);
}

std::unique_ptr<ember::Container> container;

void setup() {
  Serial.begin(115200);

  // Initialize the embers required resources.
  ember::initialize(ember::logging::LogLevel::Debug);

  // Create the main container instance.
  container = std::make_unique<ember::Container>();

  auto sender = create_sender_agent();
  auto receiver = create_receiver_agent();

  container->add_agent(std::move(sender));
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