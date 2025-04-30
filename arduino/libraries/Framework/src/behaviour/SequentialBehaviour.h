#ifndef FRAMEWORK_BEHAVIOUR_SEQUENTIALBEHAVIOUR_H
#define FRAMEWORK_BEHAVIOUR_SEQUENTIALBEHAVIOUR_H

#include <vector>
#include <memory>

#include "Behaviour.h"

#include "../Object.h"

namespace framework {

namespace behaviour {

template<class E=void>
class SequentialBehaviourQueue:
    public Object<__ffi::SequentialBehaviourQueue<__ffi::Event>> {
  public:
    SequentialBehaviourQueue();

    void add_behaviour(std::unique_ptr<Behaviour<E>>&& behaviour);
};

template<class E=void, class ChildEvent=void>
class SequentialBehaviour:
    public Behaviour<E>,
    public Object<__ffi::SequentialBehaviour> {
  public:
    SequentialBehaviour(SequentialBehaviourQueue<ChildEvent>&& initial_behaviours);
    virtual ~SequentialBehaviour();

    virtual void handle_child_event(Event<ChildEvent>&& event);

    virtual void after_child_action(Context<E>& context);

  public:
    virtual void __ffi_add_behaviour_to_agent(__ffi::Agent<__ffi::Event>* agent) override;
    virtual void __ffi_add_behaviour_to_context(
        __ffi::Context<__ffi::Event>* context,
        __ffi::ScheduleStrategy strategy
    ) override;
    virtual void __ffi_add_behaviour_to_sequential_behaviour_queue(
        __ffi::SequentialBehaviourQueue<__ffi::Event>* queue
    ) override;
};

// ======================= Impl =======================

template<class E>
SequentialBehaviourQueue<E>::SequentialBehaviourQueue():
    Object(__ffi::behaviour_sequential_queue_new(), __ffi::behaviour_sequential_queue_free) {}

template<class E>
void SequentialBehaviourQueue<E>::add_behaviour(std::unique_ptr<Behaviour<E>>&& behaviour_) {
    Behaviour<E>* behaviour = behaviour_.release();
    behaviour->__ffi_add_behaviour_to_sequential_behaviour_queue(this->object);
}

template<class E, class ChildEvent>
SequentialBehaviour<E, ChildEvent>::SequentialBehaviour(SequentialBehaviourQueue<ChildEvent>&& initial_behaviours):
    Object (
        __ffi::behaviour_sequential_new(
            this,
            initial_behaviours.move_object(),
            [](void* self_, __ffi::Event* event_) {
                SequentialBehaviour<E, ChildEvent>* self = static_cast<SequentialBehaviour<E, ChildEvent>*>(self_);
                Event<ChildEvent> event(event_);
                self->handle_child_event(std::move(event));
            },
            [](void* self_, __ffi::Context<__ffi::Event>* context_) {
                SequentialBehaviour<E>* self = static_cast<SequentialBehaviour<E>*>(self_);
                Context<E> context(context_);
                self->after_child_action(context);
            }
        ),
        __ffi::behaviour_sequential_free
    ) {}

template<class E, class ChildEvent>
SequentialBehaviour<E, ChildEvent>::~SequentialBehaviour() {}

template<class E, class ChildEvent>
void SequentialBehaviour<E, ChildEvent>::handle_child_event(Event<ChildEvent>&&) {
    // Does nothing.
}

template<class E, class ChildEvent>
void SequentialBehaviour<E, ChildEvent>::after_child_action(Context<E>& context) {
    // Does nothing.
}

template<class E, class ChildEvent>
void SequentialBehaviour<E, ChildEvent>::__ffi_add_behaviour_to_agent(__ffi::Agent<__ffi::Event>* agent) {
    __ffi::agent_add_behaviour_sequential(
        agent,
        this->move_object()
    );
}

template<class E, class ChildEvent>
void SequentialBehaviour<E, ChildEvent>::__ffi_add_behaviour_to_context(
    __ffi::Context<__ffi::Event>* context,
    __ffi::ScheduleStrategy strategy
) {
    __ffi::context_insert_behaviour_sequential(
        context,
        this->move_object(),
        strategy
    );
}

template<class E, class ChildEvent>
void SequentialBehaviour<E, ChildEvent>::__ffi_add_behaviour_to_sequential_behaviour_queue(
    __ffi::SequentialBehaviourQueue<__ffi::Event>* queue
) {
    __ffi::behaviour_sequential_queue_add_behaviour_sequential(
        queue,
        this->move_object()
    );
}

} // namespace behaviour

} // namespace framework

#endif // FRAMEWORK_BEHAVIOUR_SEQUENTIALBEHAVIOUR_H
