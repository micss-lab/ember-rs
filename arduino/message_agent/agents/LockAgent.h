#ifdef USE_EMBER

#ifndef LOCK_AGENT_H
#define LOCK_AGENT_H

#include "Ember.h"

#include "./PirAgent.h"

namespace agents::lock {

namespace ontology {

const char* lock_ontology() {
    return "Lock-Ontology";
}

enum class LockActionValue {
    Lock,
    Unlock,
};

struct LockAction {
    LockActionValue value;

    static LockAction unlock() {
        return LockAction {.value = LockActionValue::Unlock};
    }

    static LockAction lock() {
        return LockAction {.value = LockActionValue::Lock};
    }

    static LockAction decode_message(const ember::message::Message& message) {
        LockAction lock_action{};
        ember::message::ContentView content = message.get_content();
        memcpy(&lock_action, content.data, sizeof(LockAction));
        return lock_action;
    }

    ember::message::Message into_message() const {
        std::vector<uint8_t> content{sizeof(LockAction)};
        memcpy(content.data(), this, sizeof(LockAction));
        return ember::message::Message(ember::message::Performative::Inform, {"control@local"}, lock_ontology(), content);
    }
};

} // namespace ontology

struct LockState {
   bool locked{true};
   const char* password;

   bool object_detected{false};
   
    void unlock() {
        Serial.println("Unlocking door, enter password:");

        uint8_t password_input[25];
        read_chars_from_uart(password_input, 25);

        Serial.print("Password: ");
        Serial.println((char*)password_input);

        if (memcmp(password_input, password, strlen(password)) == 0) {
            Serial.println("Password correct, unlocking!");
            locked = false;
        } else {
            Serial.print("password: ");
            Serial.println((char*)password_input);
            Serial.print("set password: ");
            Serial.println((char*)password);
            Serial.println("Incorrect password, door remains locked.");
        }
    }

    void lock() {
        if (!this->object_detected) {
            this->locked = true;
        }
    }
};

class UnlockButton:
    public ember::behaviour::CyclicBehaviour<LockState> {
  public:
    UnlockButton(unsigned int unlock_button_pin): 
        unlock_button_pin(unlock_button_pin) {}
    
    void action(ember::behaviour::Context<>& context, LockState& agent_state) override {
        if (digitalRead(this->unlock_button_pin) == LOW && !this->was_pressed) {
            Serial.println("[INFO] Unlock button pressed.");
            agent_state.unlock();

            if (!agent_state.locked) {
                context.send_message(ontology::LockAction::unlock().into_message().wrap_with_envelope());
            }
            this->was_pressed = true;
        }
    }

    bool is_finished() const override {
        return false;
    }

  private:
    unsigned int unlock_button_pin;

    bool was_pressed{false};
};

class AutoLock:
    public ember::behaviour::TickerBehaviour<LockState> {
  public:
    uint64_t interval_millis() const override {
        return 1;
    }

    void action(ember::behaviour::Context<>& context, LockState& agent_state) override {
        if (agent_state.locked) {
            return;
        } else if (this->unlocked_at == 0) {
            this->unlocked_at = millis();
        } else if (millis() - this->unlocked_at >= 5000) {
            Serial.println("Automatically locking door.");
            agent_state.lock();

            context.send_message(ontology::LockAction::lock().into_message().wrap_with_envelope());
        }
    }

    bool is_finished() const override {
        return false;
    }

  private:
    unsigned long unlocked_at{};
};

class PirReceiver:
    public ember::behaviour::CyclicBehaviour<LockState> {
  public:
    void action(ember::behaviour::Context<>& context, LockState& agent_state) override {
        std::optional<ember::message::Message> message = context.receive_message_with_filter(
            ember::message::MessageFilter::ontology(agents::pir::ontology::pir_ontology())
        );
        if (!message.has_value()) {
            context.block_behaviour();
            return;
        }
        auto object = agents::pir::ontology::Object::decode_message(std::move(message.value()));
        agent_state.object_detected = object.detected;
    }

    bool is_finished() const override {
        return false;
    }
};

ember::Agent<LockState> create_lock_agent(unsigned int unlock_button_pin) {
    ember::Agent<LockState> lock_agent{"lock", std::move(LockState{})};
    auto unlock_button = std::make_unique<UnlockButton>(unlock_button_pin);
    auto auto_lock = std::make_unique<AutoLock>();
    auto pir_receiver = std::make_unique<PirReceiver>();
    lock_agent.add_behaviour(std::move(unlock_button));
    lock_agent.add_behaviour(std::move(auto_lock));
    lock_agent.add_behaviour(std::move(pir_receiver));
    return std::move(lock_agent);
}

} // namesapce agents

#endif // LOCK_AGENT_H

#endif // USE_EMBER