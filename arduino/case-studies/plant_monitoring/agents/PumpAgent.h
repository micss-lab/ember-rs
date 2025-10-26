#ifdef USE_EMBER

#ifndef PUMP_AGENT_H
#define PUMP_AGENT_H

#include "Ember.h"

namespace agents::pump {

namespace ontology {

const char* pump_ontology() {
    return "Pump-Ontology";
}

enum class PumpActionValue {
    Activate,
    Deactivate,
};

struct PumpAction {

    PumpActionValue value{};

    static PumpAction activate() {
        return PumpAction {.value = PumpActionValue::Activate};
    }

    static PumpAction deactivate() {
        return PumpAction {.value = PumpActionValue::Deactivate};
    }

    static PumpAction decode_message(ember::message::Message message) {
        PumpAction pump_action{};
        ember::message::ContentView content = message.get_content();
        memcpy(&pump_action, content.data, sizeof(PumpAction));
        return pump_action;
    }

    ember::message::Message into_message() const {
        std::vector<uint8_t> content{sizeof(PumpAction)};
        memcpy(content.data(), this, sizeof(PumpAction));
        return ember::message::Message(ember::message::Performative::Request, {"pump@local"}, pump_ontology(), content);
    }

};

struct PumpStatus {
    bool active{false};
    bool changed{false};

    static PumpStatus decode_message(const ember::message::Message& message) {
        PumpStatus pump_status{};
        ember::message::ContentView content = message.get_content();
        memcpy(&pump_status, content.data, sizeof(PumpStatus));
        return pump_status;
    }

    ember::message::Message into_message() const {
        std::vector<uint8_t> content{sizeof(PumpStatus)};
        memcpy(content.data(), this, sizeof(PumpStatus));
        return ember::message::Message(ember::message::Performative::Inform, {"control@local"}, pump_ontology(), content);
    }
};

} // namespace ontology

struct PumpState {
    bool active{false};
};

class PumpInteractions:
    public ember::behaviour::CyclicBehaviour<PumpState> {
  public:
    PumpInteractions() = default;

    void action(ember::behaviour::Context<>& context, PumpState& agent_state) override {
        std::optional<ember::message::Message> message = context.receive_message();
        if (!message.has_value()) {
            context.block_behaviour();
            return;
        }


        auto action = ontology::PumpAction::decode_message(std::move(message.value())).value;
        if (action == ontology::PumpActionValue::Activate && !agent_state.active) {
            agent_state.active = true;
        } else if (action == ontology::PumpActionValue::Deactivate && agent_state.active) {
            agent_state.active = false;
        } else {
            context.send_message(std::move(
                ontology::PumpStatus {
                    .active = agent_state.active,
                    .changed = false
                }.into_message().wrap_with_envelope()
            ));
            return;
        }

        Serial.print("[DEBUG] New pump state: ");
        Serial.println(agent_state.active);
        context.send_message(std::move(
            ontology::PumpStatus {
                .active = agent_state.active,
                .changed = true
            }.into_message().wrap_with_envelope()
        ));
    }

    bool is_finished() const override {
        return false;
    }
};

class PumpLight:
    public ember::behaviour::CyclicBehaviour<PumpState> {
  public:
    PumpLight(unsigned int pump_light_pin): pump_light_pin(pump_light_pin) {}

    void action(ember::behaviour::Context<>& context, PumpState& agent_state) override {
        digitalWrite(this->pump_light_pin, agent_state.active);
    }

    bool is_finished() const override {
        return false;
    }

  private:
    unsigned int pump_light_pin{};
};

ember::Agent<PumpState> create_pump_agent(unsigned int pump_light_pin) {
    ember::Agent<PumpState> pump_agent{"pump-agent", std::move(PumpState{})};
    auto sensor_behaviour = std::make_unique<PumpInteractions>();
    auto pump_alert_behaviour = std::make_unique<PumpLight>(pump_light_pin);
    pump_agent.add_behaviour(std::move(sensor_behaviour));
    pump_agent.add_behaviour(std::move(pump_alert_behaviour));
    return std::move(pump_agent);
}

} // namesapce agents

#endif // PUMP_AGENT_H

#endif // USE_EMBER