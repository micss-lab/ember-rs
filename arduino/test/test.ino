#include <Ember.h>

#include <utility>
#include <memory>

class HelloWorld: 
    public ember::behaviour::OneShotBehaviour<> {
  public:
    virtual void action(ember::behaviour::Context<>& context, ember::Unit& agent_state) const override {
      Serial.println("Hello, World!");
      Serial.println("My friend will print 10 messages.");
    }
};

class MessagePrinter:
    public ember::behaviour::CyclicBehaviour<> {
  public:
    virtual void action(ember::behaviour::Context<>& context, ember::Unit& agent_state) override {
        Serial.println(
          (this->count == 10)
            ? "Printing the first message!"
            : (
              (this->count == 1)
                ? "Printing the last message!"
                : "Printing another message."
            )
        );
        --this->count;
    }
    virtual bool is_finished() const override {
      return this->count == 0;
    }
  private:
    unsigned int count{10};
};

std::unique_ptr<ember::Container> container;

void setup() {
  Serial.begin(115200);
  Serial.println("Hello, ESP32-C3!");

  // Initialize the embers required resources.
  ember::initialize(ember::logging::LogLevel::Debug);
  /* ember::__ffi::initialize_logging(5); */
  /* ember::__ffi::initialize_allocator(); */

  // Create the main container instance.
  container = std::make_unique<ember::Container>();
  /* ember::__ffi::Container* container = ember::__ffi::container_new(); */

  ember::Agent<> hello_world_agent("hello-world-agent", ember::Unit());
  hello_world_agent.add_behaviour(std::make_unique<HelloWorld>());
  hello_world_agent.add_behaviour(std::make_unique<MessagePrinter>());

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
