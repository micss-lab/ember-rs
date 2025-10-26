#ifdef USE_EMBER

#ifndef LIGHT_AGENT_H
#define LIGHT_AGENT_H

#include "Ember.h"

namespace agents::light {

namespace ontology {

const char* light_ontology() {
    return "Light-Ontology";
}

struct LightLevel {
    float lux{0};

    static LightLevel decode_message(const ember::message::Message& message) {
        LightLevel light_level{};
        ember::message::ContentView content = message.get_content();
        memcpy(&light_level, content.data, sizeof(LightLevel));
        return light_level;
    }

    ember::message::Message into_message() const {
        std::vector<uint8_t> content{sizeof(LightLevel)};
        memcpy(content.data(), this, sizeof(LightLevel));
        return ember::message::Message(ember::message::Performative::Inform, {"control@local"}, light_ontology(), content);
    }
};

} // namespace ontology

struct LightState {
    float lux{0};
};

class SensorBehaviour:
    public ember::behaviour::TickerBehaviour<LightState> {
  public:
    SensorBehaviour(unsigned int ldr_sensor_pin): 
        ldr_sensor_pin(ldr_sensor_pin) {}
    
    uint64_t interval_millis() const override {
        return 100;
    }

    void action(ember::behaviour::Context<>& context, LightState& agent_state) override {
        int rawLight = analogRead(LDR_PIN);
        float sensorLux = ((4095 - rawLight) / 4095.0) * (MAX_LUX - MIN_LUX) + MIN_LUX;
        int mappedLuxGauge = (int)(((sensorLux - MIN_LUX) / (MAX_LUX - MIN_LUX)) * 4095);

        agent_state.lux = mappedLuxGauge;
    }

    bool is_finished() const override {
        return false;
    }

  private:
    unsigned int ldr_sensor_pin;
};

class LightAlertBehaviour:
    public ember::behaviour::TickerBehaviour<LightState> {
  public:
    LightAlertBehaviour(unsigned int light_alert_pin): light_alert_pin(light_alert_pin) {}

    uint64_t interval_millis() const override {
        return 250;
    }

    void action(ember::behaviour::Context<>& context, LightState& agent_state) override {
        digitalWrite(light_alert_pin, agent_state.lux < LIGHT_ALERT_THRESHOLD);
    }

    bool is_finished() const override {
        return false;
    }

  private:
    unsigned int light_alert_pin{};
};

ember::Agent<LightState> create_light_agent(unsigned int ldr_sensor_pin, unsigned int light_alert_pin) {
    ember::Agent<LightState> light_agent{"light", std::move(LightState{})};
    std::unique_ptr<SensorBehaviour> sensor_behaviour = std::make_unique<SensorBehaviour>(ldr_sensor_pin);
    std::unique_ptr<LightAlertBehaviour> light_alert_behaviour = std::make_unique<LightAlertBehaviour>(light_alert_pin);
    light_agent.add_behaviour(std::move(sensor_behaviour));
    light_agent.add_behaviour(std::move(light_alert_behaviour));
    return std::move(light_agent);
}

} // namesapce agents

#endif // LIGHT_AGENT_H

#endif // USE_EMBER