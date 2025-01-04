#include "SequentialBehaviour.h"

using namespace framework::behaviour;

SequentialBehaviourQueue::SequentialBehaviourQueue():
    Object(__ffi::behaviour_sequential_queue_new(), __ffi::behaviour_sequential_queue_free) {}

// template<class Message>
// void SequentialBehaviourQueue::add_behaviour(std::unique_ptr<Behaviour<Message>>&& behaviour_) {}
