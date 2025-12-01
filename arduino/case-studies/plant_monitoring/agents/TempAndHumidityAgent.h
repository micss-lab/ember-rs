#define USE_EMBER

#ifdef USE_EMBER

#ifndef TEMP_AND_HUMIDITY_AGENT_H
#define TEMP_AND_HUMIDITY_AGENT_H

#include <array>

#include "Ember.h"

namespace agents::temp_and_humidity {

namespace ontology {

const char* temp_and_humidity_ontology() {
    return "Light-And-Humidity-Ontology";
}

struct Measurement {
    float temperature{0};
    float humidity{0};

    static Measurement decode_message(const ember::message::Message& message) {
        Measurement measurement{};
        ember::message::ContentView content = message.get_content();
        
        // Deserialize each field individually
        memcpy(&measurement.temperature, content.data, sizeof(float));
        memcpy(&measurement.humidity, content.data + sizeof(float), sizeof(float));
        
        return measurement;
    }

    ember::message::Message into_message() const {
        // Serialize each field individually (total size: 2 floats)
        std::vector<uint8_t> content(2 * sizeof(float));
        memcpy(content.data(), &this->temperature, sizeof(float));
        memcpy(content.data() + sizeof(float), &this->humidity, sizeof(float));
        
        return ember::message::Message(
            ember::message::Performative::Inform, 
            {"control@local"}, 
            temp_and_humidity_ontology(), 
            content
        );
    }
};

} // namespace ontology

class SensorBehaviour:
    public ember::behaviour::TickerBehaviour<> {
  public:
    uint64_t interval_millis() const override {
        return 3000;
    }

    void action(ember::behaviour::Context<>& context, ember::Unit& agent_state) override {
        context.send_message(this->measurements[this->current++].into_message().wrap_with_envelope());
        if (this->current == this->measurements.size()) {
            this->current = 0;
        }
    }

    bool is_finished() const override {
        return false;
    }

  private:
    std::array<ontology::Measurement, 10> measurements{{}};
    unsigned int current{0};
};


ember::Agent<> create_temp_and_humidity_agent() {
    ember::Agent<> light_agent{"temp-and-humidity", std::move(ember::Unit{})};
    return std::move(light_agent);
}

} // namesapce agents

#endif // TEMP_AND_HUMIDITY_AGENT_H

#endif // USE_EMBER