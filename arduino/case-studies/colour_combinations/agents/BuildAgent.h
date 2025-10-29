#define USE_EMBER

#ifdef USE_EMBER

#ifndef BUILDER_AGENT_H
#define BUILDER_AGENT_H

#include "Ember.h"

namespace agents::builder {

namespace ontology {

const char* builder_ontology() {
    return "Builder-Ontology";
}

struct BuildMessage {
    static BuildMessage decode_message(const ember::message::Message& message) {
        // In the simple case, BuildMessage has no fields, so just return an instance
        return BuildMessage{};
    }

    ember::message::Message into_message() const {
        // Empty content since BuildMessage has no fields
        std::vector<uint8_t> content{};
        return ember::message::Message(
            ember::message::Performative::Request, 
            {"builder@local"}, 
            builder_ontology(), 
            content
        );
    }
};

} // namespace ontology

class BuildBehaviour:
    public ember::behaviour::CyclicBehaviour<std::shared_ptr<Belt>> {
  public:
    void action(ember::behaviour::Context<>& context, std::shared_ptr<Belt>& agent_state) override {
        std::optional<ember::message::Message> message = context.receive_message();
        
        if (!message.has_value()) {
            context.block_behaviour();
            return;
        }

        ontology::BuildMessage build_msg = ontology::BuildMessage::decode_message(std::move(message.value()));
        
        auto next_colour = agent_state->take_next();
        if (!next_colour.has_value()) {
            // Log error: "no item found when expecting to build"
            return;
        }

        this->store(*next_colour, agent_state);
    }

    bool is_finished() const override {
        return false;
    }

  private:
    std::optional<Colour> stored;

    void store(Colour colour, std::shared_ptr<Belt>& state) {
        if (this->stored.has_value()) {
            Colour stored_colour = *this->stored;
            this->stored = std::nullopt;
            
            int score = state->made_combination(stored_colour, colour);
            
            // Log: "Storing [colour] brick on top"
            // Log: "Combining [stored_colour] and [colour] for a score of [score]"
            Serial.print("Storing brick on top\n");
            Serial.print("Combining bricks for a score of ");
            Serial.println(score);
        } else {
            // Log: "Storing [colour] brick"
            Serial.println("Storing brick");
            this->stored = colour;
        }
    }
};

ember::Agent<std::shared_ptr<Belt>> create_builder_agent(std::shared_ptr<Belt> belt) {
    ember::Agent<std::shared_ptr<Belt>> builder_agent{"builder", std::move(belt)};
    auto build_behaviour = std::make_unique<BuildBehaviour>();
    builder_agent.add_behaviour(std::move(build_behaviour));
    return std::move(builder_agent);
}

} // namespace agents::builder

#endif // BUILDER_AGENT_H

#endif // USE_EMBER