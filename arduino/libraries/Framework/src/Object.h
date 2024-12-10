#ifndef FRAMEWORK_OBJECT_H
#define FRAMEWORK_OBJECT_H

#include <functional>

namespace framework {

/**
 * Internal type used for managing ownership of ffi pointers.
 */
template<class T>
class Object {
  public:
    T* move_object();

  protected:
    Object(T* object, std::function<void(T*)> free);
    ~Object();

  protected:
    bool moved{false};
    T* value;
    std::function<void(T*)> free;
};

template<class T>
Object<T>::Object(T* value, std::function<void(T*)>):
    value(value), free(free) {}

template<class T>
T* Object<T>::move_object() {
    if (this->moved) {
        throw "Use of moved value.";
    }

    this->moved = true;
    return this->value;
}

template<class T>
Object<T>::~Object() {
    if (this->moved) {
        return;
    }
    this->free(this->value);
}

} // namespace framework

#endif // FRAMEWORK_OBJECT_H
