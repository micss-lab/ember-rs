#include <Framework.h>

#include <utility>

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

void setup() {
  Serial.begin(115200);
  Serial.println("Hello, ESP32-C3!");

  // Initialize the frameworks required resources.
  framework::initialize(framework::logging::LogLevel::Off);
  /* framework::__ffi::initialize_logging(5); */
  /* framework::__ffi::initialize_allocator(); */

  // Create the main container instance.
  framework::Container container{};
  /* framework::__ffi::Container* container = framework::__ffi::container_new(); */

  framework::Agent hello_world_agent = framework::Agent("hello-world-agent");
  hello_world_agent.add_behaviour(std::make_unique<HelloWorld>());
  hello_world_agent.add_behaviour(std::make_unique<MessagePrinter>());

  container.add_agent(std::move(hello_world_agent));

  // Start the container and check for errors.
  bool failed = framework::Container::start(std::move(container));
  /* int failed = (bool) framework::__ffi::container_start(container); */
  if (failed) {
    Serial.println("Container exited with an error!");
  }

  Serial.println("Finished executing.");
}

void loop() {

}