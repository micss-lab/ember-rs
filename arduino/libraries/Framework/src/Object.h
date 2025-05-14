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

    // Do not allow to copy an object.
    Object(const Object<T>&) = delete;
    Object<T>& operator=(const Object<T>&) = delete;

    Object(Object<T>&&);
    Object<T>& operator=(Object<T>&&);

  protected:
    bool moved{false};
    T* object;
    std::function<void(T*)> free;
};

template<class T>
Object<T>::Object(T* object, std::function<void(T*)> free):
    object(object), free(free) {}

template<class T>
T* Object<T>::move_object() {
    if (this->moved) {
        throw "Use of moved value.";
    }

    this->moved = true;
    return this->object;
}

template<class T>
Object<T>::Object(Object<T>&& o) {
    if (!o.moved) {
        this->object = o.move_object();
        this->free = o.free;
    }
}

template<class T>
Object<T>& Object<T>::operator=(Object<T>&& o) {
    if (this != &o && !o.moved) {
        this->object = o.move_object();
        this->free = o.free;
        this->moved = false;
    }
    return *this;
}

template<class T>
Object<T>::~Object() {
    if (this->moved) {
        return;
    }
    this->free(this->object);
}

} // namespace framework

#endif // FRAMEWORK_OBJECT_H
