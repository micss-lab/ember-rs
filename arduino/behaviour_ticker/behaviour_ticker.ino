#include <Framework.h>

#include <utility>
#include <memory>

class HelloWorld: 
    public framework::behaviour::OneShotBehaviour<> {
  public:
    virtual void action(framework::behaviour::Context<>& context) const override {
      Serial.println("Hello, World!");
      Serial.println("My friend will print 10 messages with a longer interval each time.");
    }
};

class MessagePrinter:
    public framework::behaviour::TickerBehaviour<> {
  public:
    virtual uint64_t interval_millis() const override {
      return 200 * (this->start - (this->count + 1));
    }
    virtual void action(framework::behaviour::Context<>& context) override {
        Serial.println("Does this really work?");
        --this->count;
    }
    virtual bool is_finished() const override {
      return this->count == 0;
    }
  private:
    unsigned int start{10};
    unsigned int count{10};
};

class Confirmation:
    public framework::behaviour::OneShotBehaviour<> {
  public:
    virtual void action(framework::behaviour::Context<>& context) const override {
      Serial.println("Yes it does!");
    }
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
  hello_world_agent.add_behaviour(std::make_unique<Confirmation>());

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