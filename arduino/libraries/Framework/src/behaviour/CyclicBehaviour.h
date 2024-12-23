#ifndef FRAMEWORK_BEHAVIOUR_CYCLICBEHAVIOUR_H
#define FRAMEWORK_BEHAVIOUR_CYCLICBEHAVIOUR_H

#include "Behaviour.h"

#include "../Object.h"

namespace framework {

namespace behaviour {

class CyclicBehaviour:
    public Behaviour,
    public Object<__ffi::CyclicBehaviour> {
  public:
    CyclicBehaviour();
    virtual ~CyclicBehaviour();

    virtual void action(Context& context) = 0;
    virtual bool is_finished() const = 0;

    virtual void __ffi_add_behaviour_to_agent(__ffi::Agent<__ffi::Message>* agent) override;
};

} // namespace behaviour

} // namespace framework

#endif // FRAMEWORK_BEHAVIOUR_CYCLICBEHAVIOUR_H
