#include <Framework.h>

#include <utility>

class HelloWorld: 
  public framework::behaviour::OneShotBehaviour {
  public:
    virtual void action(framework::behaviour::Context& context) {
      Serial.println("Hello, World!");
    }
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