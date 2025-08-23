#include <Ember.h>

#include <utility>
#include <iostream>
#include <memory>

using namespace ember::behaviour;

class HelloWorld: 
    public OneShotBehaviour<> {
  public:
    virtual void action(Context<>& context, ember::Unit& agent_state) const override {
      Serial.println("Hello, World!");
      Serial.println("My friends will send messages to their parent to calculate a value.");
    }
};

class Incrementor:
    public CyclicBehaviour<ember::Unit, char> {
  public:
    virtual void action(Context<char>& context, ember::Unit& agent_state) override {
        context.emit_event(Event(std::make_unique<char>(this->count)));
        --this->count;
    }
    virtual bool is_finished() const override {
      return this->count == 0;
    }
  private:
    unsigned int count{10};
};

class SomethingSequential:
    public SequentialBehaviour<ember::Unit, void, char> {
  public:
    SomethingSequential():
     SequentialBehaviour<ember::Unit, void, char>(
       SomethingSequential::initial_behaviours()
     ) {}

    static BehaviourVec<ember::Unit, char> initial_behaviours() {
      BehaviourVec<ember::Unit, char> vec;
      // vec.add_behaviour(std::make_unique<HelloWorld>());
      vec.add_behaviour(std::make_unique<Incrementor>());
      vec.add_behaviour(std::make_unique<Incrementor>());
      return vec;
    }

    void handle_child_message(Event<char>&& event) {
      char value = *(event.value());

      Serial.print("Receiving child message: ");
      Serial.println((unsigned int) value);

      this->count += value;

      Serial.print("Calculated a value of `");
      Serial.print(this->count);
      Serial.println("` so far...");
    }

  private:
    unsigned int count{0};
};

std::unique_ptr<ember::Container> container;

void setup() {
  Serial.begin(115200);
  Serial.println("Running example `message_parent`");

  // Initialize the embers required resources.
  ember::initialize(ember::logging::LogLevel::Debug);
  // ember::__ffi::initialize_logging(5);
  /* ember::__ffi::initialize_allocator(); */

  // Create the main container instance.
  container = std::make_unique<ember::Container>();
  /* ember::__ffi::Container* container = ember::__ffi::container_new(); */

  ember::Agent<> hello_world_agent("hello-world-agent", ember::Unit());
  hello_world_agent.add_behaviour(std::make_unique<HelloWorld>());
  hello_world_agent.add_behaviour(std::make_unique<SomethingSequential>());

  container->add_agent(std::move(hello_world_agent));

  Serial.println("Finished setup");
}

void loop() {
  ember::Container::PollResult result = container->poll();
  if (result.should_stop) {
    Serial.println(
      (result.status == 0) ? "Finished executing." : "Container exited with an error!"
    );
    exit(result.status);
  }
}
