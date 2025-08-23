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

} // namespace behaviour

} // namespace ember

#endif // EMBER_BEHAVIOUR_EVENT_H
