#define USE_EMBER

#ifdef USE_EMBER

#ifndef SORT_AGENT_H
#define SORT_AGENT_H

#include "Ember.h"

#include "./BuildAgent.h"
#include "./TrashAgent.h"

namespace agents::sort {

// Forward declarations for the behaviour classes
class Empty;
class Red;
class Green;
class Blue;

template<class E>
void build(ember::behaviour::Context<E>& ctx) {
    auto msg = agents::builder::ontology::BuildMessage{}.into_message().wrap_with_envelope();
    ctx.send_message(std::move(msg));
}

template<class E>
void trash(ember::behaviour::Context<E>& ctx) {
    auto msg = agents::trasher::ontology::TrashMessage{}.into_message().wrap_with_envelope();
    ctx.send_message(std::move(msg));
}

class Empty:
    public ember::behaviour::CyclicBehaviour<
        std::shared_ptr<Belt>,
        ember::behaviour::FsmEvent<>
    > {
  public:
    void action(
        ember::behaviour::Context<ember::behaviour::FsmEvent<>>& context,
        std::shared_ptr<Belt>& agent_state
    ) override {
        auto window = agent_state->next_window();
        
        if (!window.has_value()) {
            agent_state->print_score();
            this->finish = true;
            context.stop_container();
            return;
        }
        
        if (!window->second.has_value()) {
            build(context);
            context.emit_event(std::move(ember::behaviour::FsmEvent<> {
                .kind = ember::behaviour::FsmEvent<>::Transition,
                .transition = colour_as_string(window->first),
            }));
            return;
        }
        
        Colour first = window->first;
        Colour second = *window->second;
        
        if (first == Colour::Red) {
            build(context);
            context.emit_event(std::move(ember::behaviour::FsmEvent<> {
                .kind = ember::behaviour::FsmEvent<>::Transition,
                .transition = colour_as_string(Colour::Red),
            }));
        } else if ((first == Colour::Green && second == Colour::Green) ||
                   (first == Colour::Blue && second == Colour::Blue)) {
            build(context);
            context.emit_event(std::move(ember::behaviour::FsmEvent<> {
                .kind = ember::behaviour::FsmEvent<>::Transition,
                .transition = colour_as_string(first),
            }));
        } else {
            trash(context);
        }
    }

    bool is_finished() const override {
        return this->finish;
    }

  private:
    bool finish{false};
};

class Red:
    public ember::behaviour::CyclicBehaviour<
        std::shared_ptr<Belt>,
        ember::behaviour::FsmEvent<>
    > {
  public:
    void action(
        ember::behaviour::Context<ember::behaviour::FsmEvent<>>& context,
        std::shared_ptr<Belt>& agent_state
    ) override {
        auto window = agent_state->next_window();
        
        if (!window.has_value()) {
            // Log warning: "Failed to make final combination!"
            return;
        }
        
        if (window->first == Colour::Red) {
            build(context);
            this->built = true;
        } else if (window->second.has_value() && *window->second == Colour::Red) {
            trash(context);
        } else {
            build(context);
            this->built = true;
        }
    }

    bool is_finished() const override {
        return this->built;
    }

  private:
    bool built{false};
};

class Green:
    public ember::behaviour::OneShotBehaviour<
        std::shared_ptr<Belt>,
        ember::behaviour::FsmEvent<>
    > {
  public:
    void action(
        ember::behaviour::Context<ember::behaviour::FsmEvent<>>& context,
        std::shared_ptr<Belt>& agent_state
    ) const override {
        auto next = agent_state->peek_next();
        if (next.has_value()) {
            build(context);
        } else {
            // Log warning: "Failed to make final combination!"
        }
    }
};

class Blue:
    public ember::behaviour::OneShotBehaviour<
        std::shared_ptr<Belt>,
        ember::behaviour::FsmEvent<>
    > {
  public:
    void action(
        ember::behaviour::Context<ember::behaviour::FsmEvent<>>& context,
        std::shared_ptr<Belt>& agent_state
    ) const override {
        auto next = agent_state->peek_next();
        if (next.has_value()) {
            build(context);
        } else {
            // Log warning: "Failed to make final combination!"
        }
    }
};

class DecisionBehaviour:
    public ember::behaviour::FsmBehaviour<
        std::shared_ptr<Belt>
    > {
  public:
    DecisionBehaviour():
        ember::behaviour::FsmBehaviour<std::shared_ptr<Belt>>(
            DecisionBehaviour::fsm()
        ) {}
    
    static ember::behaviour::Fsm<std::shared_ptr<Belt>> fsm() {
        auto builder = ember::behaviour::FsmBuilder<std::shared_ptr<Belt>>();
        
        auto empty = std::make_unique<Empty>();
        auto red = std::make_unique<Red>();
        auto blue = std::make_unique<Blue>();
        auto green = std::make_unique<Green>();
        
        auto empty_id = builder.add_behaviour(std::move(empty), true);
        auto red_id = builder.add_behaviour(std::move(red), false);
        auto blue_id = builder.add_behaviour(std::move(blue), false);
        auto green_id = builder.add_behaviour(std::move(green), false);
        
        builder.add_transition(red_id, empty_id, std::nullopt);
        builder.add_transition(blue_id, empty_id, std::nullopt);
        builder.add_transition(green_id, empty_id, std::nullopt);
        builder.add_transition(empty_id, red_id, colour_as_string(Colour::Red));
        builder.add_transition(empty_id, blue_id, colour_as_string(Colour::Blue));
        builder.add_transition(empty_id, green_id, colour_as_string(Colour::Green));
        
        return builder.build(empty_id);
    }
};

ember::Agent<std::shared_ptr<Belt>> create_sort_agent(std::shared_ptr<Belt> belt) {
    ember::Agent<std::shared_ptr<Belt>> sort_agent{"sort", std::move(belt)};
    auto decision_behaviour = std::make_unique<DecisionBehaviour>();
    sort_agent.add_behaviour(std::move(decision_behaviour));
    return std::move(sort_agent);
}

} // namespace agents::sort

#endif // SORT_AGENT_H

#endif // USE_EMBER