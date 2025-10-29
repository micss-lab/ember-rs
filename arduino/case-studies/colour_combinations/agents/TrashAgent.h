#define USE_EMBER

#ifdef USE_EMBER

#ifndef TRASHER_AGENT_H
#define TRASHER_AGENT_H

#include "Ember.h"

namespace agents::trasher {

namespace ontology {

const char* trasher_ontology() {
    return "Trasher-Ontology";
}

struct TrashMessage {
    static TrashMessage decode_message(const ember::message::Message& message) {
        // TrashMessage has no fields, so just return an instance
        return TrashMessage{};
    }

    ember::message::Message into_message() const {
        // Empty content since TrashMessage has no fields
        std::vector<uint8_t> content{};
        return ember::message::Message(
            ember::message::Performative::Request, 
            {"trasher@local"}, 
            trasher_ontology(), 
            content
        );
    }
};

} // namespace ontology

class TrashBehaviour:
    public ember::behaviour::CyclicBehaviour<std::shared_ptr<Belt>> {
  public:
    TrashBehaviour() = default;

    void action(ember::behaviour::Context<>& context, std::shared_ptr<Belt>& agent_state) override {
        std::optional<ember::message::Message> message = context.receive_message();
        
        if (!message.has_value()) {
            context.block_behaviour();
            return;
        }

        ontology::TrashMessage trash_msg = ontology::TrashMessage::decode_message(std::move(message.value()));
        
        auto next_colour = agent_state->take_next();
        if (!next_colour.has_value()) {
            // Log error: "no item found when expecting to trash"
            Serial.println("ERROR: no item found when expecting to trash");
            return;
        }

        // Log: "Trashing [colour] brick."
        Serial.print("Trashing brick: ");
        Serial.println(static_cast<int>(*next_colour));
    }

    bool is_finished() const override {
        return false;
    }
};

ember::Agent<std::shared_ptr<Belt>> create_trasher_agent(std::shared_ptr<Belt> belt) {
    ember::Agent<std::shared_ptr<Belt>> trasher_agent{"trasher", std::move(belt)};
    auto trash_behaviour = std::make_unique<TrashBehaviour>();
    trasher_agent.add_behaviour(std::move(trash_behaviour));
    return std::move(trasher_agent);
}

} // namespace agents::trasher

#endif // TRASHER_AGENT_H

#endif // USE_EMBER