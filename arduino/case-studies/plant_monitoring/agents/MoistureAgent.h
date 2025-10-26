#ifdef USE_EMBER

#ifndef MOISTURE_AGENT_H
#define MOISTURE_AGENT_H

#include "Ember.h"

namespace agents::moisture {

namespace ontology {

const char* moisture_ontology() {
    return "Moisture-Ontology";
}

struct MoisturePercent {
    float value{};

    static MoisturePercent decode_message(ember::message::Message message) {
        MoisturePercent moisture{};
        ember::message::ContentView content = message.get_content();
        memcpy(&moisture, content.data, sizeof(MoisturePercent));
        return moisture;
    }

    ember::message::Message into_message() const {
        std::vector<uint8_t> content{sizeof(MoisturePercent)};
        memcpy(content.data(), this, sizeof(MoisturePercent));
        return ember::message::Message(ember::message::Performative::Request, {"control@local"}, moisture_ontology(), content);
    }

};

} // namespace ontology

struct MoistureState {
    float percent{};
};

class PotentionmeterSensor:
    public ember::behaviour::TickerBehaviour<MoistureState> {
  public:
    PotentionmeterSensor(unsigned int potentiometer_pin): 
        potentiometer_pin(potentiometer_pin) {};

    uint64_t interval_millis() const override {
        return 100;
    }

    void action(ember::behaviour::Context<>& context, MoistureState& agent_state) override {
        int moisture = analogRead(this->potentiometer_pin);
        float percent = ((float) moisture) / 4095.0 * 100.0;
        agent_state.percent = percent;
        context.send_message(std::move(
            ontology::MoisturePercent {
                .value = percent
            }.into_message().wrap_with_envelope()
        ));
    }

    bool is_finished() const override {
        return false;
    }

  private:
    unsigned int potentiometer_pin{};
};

ember::Agent<MoistureState> create_moisture_agent(unsigned int potentiometer_pin) {
    ember::Agent<MoistureState> moisture_agent{"moisture-agent", std::move(MoistureState{})};
    auto potentiometer_sensor = std::make_unique<PotentionmeterSensor>(potentiometer_pin);
    moisture_agent.add_behaviour(std::move(potentiometer_sensor));
    return std::move(moisture_agent);
}

} // namesapce agents

#endif // MOISTURE_AGENT_H

#endif // USE_EMBER