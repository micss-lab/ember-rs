#include <Framework.h>

#include <utility>
#include <iostream>
#include <memory>

using namespace framework::behaviour;

class HelloWorld: 
    public OneShotBehaviour<> {
  public:
    virtual void action(Context<>& context) const override {
      Serial.println("Hello, World!");
      Serial.println("My friends will print 10 messages followed by 20 of something else.");
    }
};

class Incrementor:
    public CyclicBehaviour<char> {
  public:
    virtual void action(Context<char>& context) override {
        context.message_parent(Message(std::make_unique<char>(this->count)));
        --this->count;
    }
    virtual bool is_finished() const override {
      return this->count == 0;
    }
  private:
    unsigned int count{10};
};

class SomethingSequential:
    public SequentialBehaviour<void, char> {
  public:
    SomethingSequential():
     SequentialBehaviour<void, char>(
       SomethingSequential::initial_behaviours()
     ) {}

    static SequentialBehaviourQueue<char> initial_behaviours() {
      SequentialBehaviourQueue<char> queue;
      // queue.add_behaviour(std::make_unique<HelloWorld>());
      queue.add_behaviour(std::make_unique<Incrementor>());
      queue.add_behaviour(std::make_unique<Incrementor>());
      return queue;
    }

    void handle_child_message(Message<char>&& message) {
      char value = *(message.value());

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

std::unique_ptr<framework::Container> container;

void setup() {
  Serial.begin(115200);
  Serial.println("Running example `message_parent`");

  // Initialize the frameworks required resources.
  framework::initialize(framework::logging::LogLevel::Debug);
  // framework::__ffi::initialize_logging(5);
  /* framework::__ffi::initialize_allocator(); */

  // Create the main container instance.
  container = std::make_unique<framework::Container>();
  /* framework::__ffi::Container* container = framework::__ffi::container_new(); */

  framework::Agent hello_world_agent = framework::Agent("hello-world-agent");
  hello_world_agent.add_behaviour(std::make_unique<SomethingSequential>());

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