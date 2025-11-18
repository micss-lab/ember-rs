#define USE_EMBER

#ifdef USE_EMBER

#ifndef PIR_AGENT_H
#define PIR_AGENT_H

#include "Ember.h"

namespace agents::pir {

namespace ontology {

const char* pir_ontology() {
    return "Pir-Ontology";
}

struct Object {
    bool detected{};

    static Object decode_message(const ember::message::Message& message) {
        Object object{};
        ember::message::ContentView content = message.get_content();
        memcpy(&object, content.data, sizeof(Object));
        return object;
    }

    ember::message::Message into_message() const {
        std::vector<uint8_t> content{sizeof(Object)};
        memcpy(content.data(), this, sizeof(Object));
        return ember::message::Message(ember::message::Performative::Inform, {"lock@local"}, pir_ontology(), content);
    }
};

} // namespace ontology

class PirSensor:
    public ember::behaviour::CyclicBehaviour<> {
  public:
    PirSensor(unsigned int pir_sensor_pin): 
        pir_sensor_pin(pir_sensor_pin) {}
        
    void action(ember::behaviour::Context<>& context, ember::Unit& agent_state) override {
        if ((digitalRead(this->pir_sensor_pin) == HIGH) == this->object_detected) {
            return;
        }

        this->object_detected = digitalRead(this->pir_sensor_pin) == HIGH;
        context.send_message(ontology::Object {.detected = this->object_detected}.into_message().wrap_with_envelope());
    }

    bool is_finished() const override {
        return false;
    }

  private:
    unsigned int pir_sensor_pin{};

    bool object_detected{false};
};

ember::Agent<> create_pir_agent(unsigned int pir_sensor_pin) {
    ember::Agent<> pir_agent{"pir", std::move(ember::Unit{})};
    auto pir_sensor = std::make_unique<PirSensor>(pir_sensor_pin);
    pir_agent.add_behaviour(std::move(pir_sensor));
    return std::move(pir_agent);
}

} // namesapce agents

#endif // PIR_AGENT_H

#endif // USE_EMBER