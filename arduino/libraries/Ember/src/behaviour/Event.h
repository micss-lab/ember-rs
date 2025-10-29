#ifndef EMBER_BEHAVIOUR_EVENT_H
#define EMBER_BEHAVIOUR_EVENT_H

#include <memory>

#include "../EmberCore.h"
#include "../Object.h"

namespace ember {

namespace behaviour {

template<class E>
class Event:
    public Object<__ffi::Event> {
  public:
    Event(std::unique_ptr<E>&& event);

    E* value();

  public:
    Event(__ffi::Event* event);
};

template<class E=Unit>
struct FsmEvent {
    enum {Transition, ChildEvent} kind;
    union {
        const char* transition;
        Event<E> event;
    };

    ~FsmEvent();

    __ffi::FsmEvent<const char*, __ffi::Event>* __ffi_into_fsm_event();
};

// Trait to detect FsmEvent
template<class T>
struct is_fsm_event : std::false_type {};

template<class E>
struct is_fsm_event<FsmEvent<E>> : std::true_type {};

template<class ChildEvent>
class Event<FsmEvent<ChildEvent>>:
    public Object<__ffi::FsmEvent<const char*, __ffi::Event>> {
  public:
    Event(FsmEvent<ChildEvent>&& event);

//   public:
//     Event(__ffi::FsmEvent<const char*, __ffi::Event>* event);
};

// ======================= Event Impl =======================

template<class E>
Event<E>::Event(std::unique_ptr<E>&& event):
    Object(
        __ffi::event_new(event.release()),
        __ffi::event_free
    ) {}

template<class E>
E* Event<E>::value() {
    E* event = static_cast<E*>(this->object->inner);
    return event;
}

template<class E>
Event<E>::Event(__ffi::Event* event):
    Object(
        event,
        __ffi::event_free
    ) {}

// ======================= FsmEvent impl =======================

template<class E>
FsmEvent<E>::~FsmEvent() {
    if (this->kind == ChildEvent) {
        this->event.~Event<E>();
    }
    // transition is a pointer, no cleanup needed
}

template<class E>
__ffi::FsmEvent<const char*, __ffi::Event>* FsmEvent<E>::__ffi_into_fsm_event() {
    switch (this->kind) {
        case FsmEvent<E>::Transition:
            return __ffi::fsm_event_transition_new(this->transition);
        case FsmEvent<E>::ChildEvent:
            return __ffi::fsm_event_event_new(this->event.move_object());
    }
}

// ======================= Event FsmEvent spec Impl =======================

template<class ChildEvent>
Event<FsmEvent<ChildEvent>>::Event(FsmEvent<ChildEvent>&& event):
    Object(
        event.__ffi_into_fsm_event(),
        __ffi::fsm_event_free
    ) {}

} // namespace behaviour

} // namespace ember

#endif // EMBER_BEHAVIOUR_EVENT_H
