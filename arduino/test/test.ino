#include <Framework.h>

#include <utility>
#include <memory>

class HelloWorld: 
    public framework::behaviour::OneShotBehaviour {
  public:
    virtual void action(framework::behaviour::Context& context) const override {
      Serial.println("Hello, World!");
      Serial.println("My friend will print 10 messages.");
    }
};

class MessagePrinter:
    public framework::behaviour::CyclicBehaviour {
  public:
    virtual void action(framework::behaviour::Context& context) override {
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

std::unique_ptr<framework::Container> container;

void setup() {
  Serial.begin(115200);
  Serial.println("Hello, ESP32-C3!");

  // Initialize the frameworks required resources.
  framework::initialize(framework::logging::LogLevel::Debug);
  /* framework::__ffi::initialize_logging(5); */
  /* framework::__ffi::initialize_allocator(); */

  // Create the main container instance.
  container = std::make_unique<framework::Container>();
  /* framework::__ffi::Container* container = framework::__ffi::container_new(); */

  framework::Agent hello_world_agent = framework::Agent("hello-world-agent");
  hello_world_agent.add_behaviour(std::make_unique<HelloWorld>());
  hello_world_agent.add_behaviour(std::make_unique<MessagePrinter>());

  container->add_agent(std::move(hello_world_agent));
  Serial.println("Finished setup");
}

void loop() {
  framework::Container::PollResult result = container->poll();
  if (result.should_stop) {
    Serial.println(
      (result.status == 0) ? "Finished executing." : "Container exited with an error!"
    );
    exit(result.status);
  }
}