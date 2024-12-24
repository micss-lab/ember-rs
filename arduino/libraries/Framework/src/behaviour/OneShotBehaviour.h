#ifndef FRAMEWORK_BEHAVIOUR_ONESHOTBEHAVIOUR_H
#define FRAMEWORK_BEHAVIOUR_ONESHOTBEHAVIOUR_H

#include "Behaviour.h"

#include "../Object.h"

namespace framework {

namespace behaviour {

class OneShotBehaviour:
    public Behaviour,
    public Object<__ffi::OneShotBehaviour> {
  public:
    OneShotBehaviour();
    virtual ~OneShotBehaviour();

    virtual void action(Context& context) const = 0;

  public:
    virtual void __ffi_add_behaviour_to_agent(__ffi::Agent<__ffi::Message>* agent) override;

    virtual void __ffi_add_behaviour_to_sequential_behaviour_queue(
        __ffi::SequentialBehaviourQueue<__ffi::Message>* queue
    ) override;
};

} // namespace behaviour

} // namespace framework

#endif // FRAMEWORK_BEHAVIOUR_ONESHOTBEHAVIOUR_H
