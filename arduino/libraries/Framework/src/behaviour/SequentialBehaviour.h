#ifndef FRAMEWORK_BEHAVIOUR_SEQUENTIALBEHAVIOUR_H
#define FRAMEWORK_BEHAVIOUR_SEQUENTIALBEHAVIOUR_H

#include <vector>
#include <memory>

#include "Behaviour.h"

#include "../Object.h"

namespace framework {

namespace behaviour {

class SequentialBehaviourQueue:
    public Object<__ffi::SequentialBehaviourQueue<__ffi::Message>> {
  public:
    SequentialBehaviourQueue();

    void add_behaviour(std::unique_ptr<Behaviour>&& behaviour);
};

class SequentialBehaviour:
    public Behaviour,
    public Object<__ffi::SequentialBehaviour> {
  public:
    SequentialBehaviour(SequentialBehaviourQueue&& initial_behaviours);
    virtual ~SequentialBehaviour();

    virtual void after_child_action(Context& context);

  public:
    virtual void __ffi_add_behaviour_to_agent(__ffi::Agent<__ffi::Message>* agent) override;

    virtual void __ffi_add_behaviour_to_sequential_behaviour_queue(
        __ffi::SequentialBehaviourQueue<__ffi::Message>* queue
    ) override;
};

} // namespace behaviour

} // namespace framework

#endif // FRAMEWORK_BEHAVIOUR_SEQUENTIALBEHAVIOUR_H
