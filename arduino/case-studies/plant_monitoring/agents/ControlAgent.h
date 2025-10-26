#ifdef USE_EMBER

#ifndef CONTROL_AGENT_H
#define CONTROL_AGENT_H

#include "Ember.h"

#include "./PumpAgent.h"
#include "./MoistureAgent.h"
#include "./LightAgent.h"
#include "./TempAndHumidityAgent.h"

namespace agents::control {

struct ControlState {
    float temperature{};
    float humidity{};
    float moisture{};
    bool pump_active{};
    float light{};

    void handle_moisture_measurement(float moisture) {
        this->moisture = moisture;
    }

    void handle_light_measurement(float light) {
        this->light = light;
    }

    void handle_pump_status_update(agents::pump::ontology::PumpStatus pump_status) {
        if (!pump_status.changed) {
            Serial.println("[WARN] Pump already in requested state.");
        } else if (pump_status.active) {
            Serial.println("[INFO] Pump successfully activated.");
        } else {
            Serial.println("[INFO] Pump successfully deactivated.");
        }
        this->pump_active = pump_status.active;
    }

    void handle_temp_and_humidity_measurement(agents::temp_and_humidity::ontology::Measurement measurement) {
        this->temperature = measurement.temperature;
        this->humidity = measurement.humidity;
    }
};

class PumpController:
    public ember::behaviour::TickerBehaviour<ControlState> {
  public:
    PumpController(unsigned int user_switch_pin): 
        user_switch_pin(user_switch_pin) {};

    uint64_t interval_millis() const override {
        return 250;
    }

    void action(ember::behaviour::Context<>& context, ControlState& agent_state) override {
        bool pump_should_be_active = digitalRead(this->user_switch_pin) == LOW || agent_state.moisture < MOISTURE_THRESHOLD;
        if (pump_should_be_active == agent_state.pump_active) {
            return;
        }
        
        agents::pump::ontology::PumpAction action;
        if (pump_should_be_active) {
            action = agents::pump::ontology::PumpAction::activate();
        } else {
            action = agents::pump::ontology::PumpAction::deactivate();
        }
        context.send_message(action.into_message().wrap_with_envelope());
    }

    bool is_finished() const override {
        return false;
    }

  private:
    unsigned int user_switch_pin{};
};

class TempAndHumidityReceiver:
    public ember::behaviour::CyclicBehaviour<ControlState> {
  public:
    void action(ember::behaviour::Context<>& context, ControlState& agent_state) override {
        std::optional<ember::message::Message> message = context.receive_message_with_filter(
            ember::message::MessageFilter::ontology(agents::temp_and_humidity::ontology::temp_and_humidity_ontology())
        );
        if (!message.has_value()) {
            context.block_behaviour();
            return;
        }
        auto measurement = agents::temp_and_humidity::ontology::Measurement::decode_message(std::move(message.value()));
        agent_state.handle_temp_and_humidity_measurement(measurement);
    }

    bool is_finished() const override {
        return false;
    }
};

class MoistureReceiver:
    public ember::behaviour::CyclicBehaviour<ControlState> {
  public:
    void action(ember::behaviour::Context<>& context, ControlState& agent_state) override {
        std::optional<ember::message::Message> message = context.receive_message_with_filter(
            ember::message::MessageFilter::ontology(agents::moisture::ontology::moisture_ontology())
        );
        if (!message.has_value()) {
            context.block_behaviour();
            return;
        }
        auto moisture = agents::moisture::ontology::MoisturePercent::decode_message(std::move(message.value()));
        agent_state.handle_moisture_measurement(moisture.value);
    }

    bool is_finished() const override {
        return false;
    }
};

class LightReceiver:
    public ember::behaviour::CyclicBehaviour<ControlState> {
  public:
    void action(ember::behaviour::Context<>& context, ControlState& agent_state) override {
        std::optional<ember::message::Message> message = context.receive_message_with_filter(
            ember::message::MessageFilter::ontology(agents::light::ontology::light_ontology())
        );
        if (!message.has_value()) {
            context.block_behaviour();
            return;
        }
        auto light = agents::light::ontology::LightLevel::decode_message(std::move(message.value()));
        agent_state.handle_light_measurement(light.lux);
    }

    bool is_finished() const override {
        return false;
    }
};

class PumpStatusReceiver:
    public ember::behaviour::CyclicBehaviour<ControlState> {
  public:
    void action(ember::behaviour::Context<>& context, ControlState& agent_state) override {
        std::optional<ember::message::Message> message = context.receive_message_with_filter(
            ember::message::MessageFilter::ontology(agents::light::ontology::light_ontology())
        );
        if (!message.has_value()) {
            context.block_behaviour();
            return;
        }
        auto pump_status = agents::pump::ontology::PumpStatus::decode_message(std::move(message.value()));
        agent_state.handle_pump_status_update(pump_status);
    }

    bool is_finished() const override {
        return false;
    }
};

class ControlStatePrinter:
    public ember::behaviour::TickerBehaviour<ControlState> {
  public:
    uint64_t interval_millis() const {
        return 1000;
    }

    void action(ember::behaviour::Context<>& context, ControlState& agent_state) override {
        Serial.println("-----------------------------");
        Serial.print("Temperature: ");
        Serial.println(agent_state.temperature);
        Serial.print("Humidity: ");
        Serial.println(agent_state.humidity);
        Serial.print("Light: ");
        Serial.println(agent_state.light);
        Serial.print("Moisture: ");
        Serial.println(agent_state.moisture);
        Serial.print("Pump Active: ");
        Serial.println(agent_state.pump_active);
        Serial.println("-----------------------------"); 
    }

    bool is_finished() const override {
        return false;
    }
};

ember::Agent<ControlState> create_control_agent(unsigned int user_switch_pin) {
    ember::Agent<ControlState> control_agent{"control", std::move(ControlState{})};
    auto pump_controller = std::make_unique<PumpController>(user_switch_pin);
    auto moisture_receiver = std::make_unique<MoistureReceiver>();
    auto light_receiver = std::make_unique<LightReceiver>();
    auto pump_status_receiver = std::make_unique<PumpStatusReceiver>();
    auto control_state_printer = std::make_unique<ControlStatePrinter>();
    control_agent.add_behaviour(std::move(pump_controller));
    control_agent.add_behaviour(std::move(moisture_receiver));
    control_agent.add_behaviour(std::move(light_receiver));
    control_agent.add_behaviour(std::move(pump_status_receiver));
    control_agent.add_behaviour(std::move(control_state_printer));
    return std::move(control_agent);
}

} // namesapce agents

#endif // CONTROL_AGENT_H

#endif // USE_EMBER