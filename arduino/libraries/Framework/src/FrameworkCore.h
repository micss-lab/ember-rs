#ifndef FRAMEWORK_CORE_H
#define FRAMEWORK_CORE_H

#include "inttypes.h"

namespace framework::__ffi {

template<typename M = void>
struct Agent;

struct Container;

template<typename M = void>
struct Context;

struct CyclicBehaviour;

struct OneShotBehaviour;

struct SequentialBehaviour;

template<typename M = void>
struct SequentialBehaviourQueue;

struct TickerBehaviour;

struct Message {
  void *inner;
};

struct ContainerPollResult {
  int32_t status;
  bool should_stop;
};

extern "C" {

void initialize_allocator();

Message *message_new(void *message);

void message_free(Message *message);

/**
 * Creates a new container instance.
 *
 * # Safety
 *
 * The ownership of the instance is transferred to the caller. Make sure to free the memory
 * with the accompanying [`container_free`].
 */
Container *container_new();

void container_free(Container *container);

void container_add_agent(Container *container, Agent<Message> *agent);

int32_t container_start(Container *container);

ContainerPollResult container_poll(Container *container);

Agent<Message> *agent_new(const char *name);

void agent_free(Agent<Message> *agent);

void agent_add_behaviour_oneshot(Agent<Message> *agent, OneShotBehaviour *oneshot);

void agent_add_behaviour_cyclic(Agent<Message> *agent, CyclicBehaviour *cyclic);

void agent_add_behaviour_ticker(Agent<Message> *agent, TickerBehaviour *ticker);

void agent_add_behaviour_sequential(Agent<Message> *agent, SequentialBehaviour *sequential);

void context_message_parent(Context<Message> *context, Message *message);

OneShotBehaviour *behaviour_oneshot_new(void *inner, void (*action)(void*, Context<Message>*));

void behaviour_oneshot_free(OneShotBehaviour *oneshot);

CyclicBehaviour *behaviour_cyclic_new(void *inner,
                                      void (*action)(void*, Context<Message>*),
                                      bool (*is_finished)(void*));

void behaviour_cyclic_free(CyclicBehaviour *cyclic);

TickerBehaviour *behaviour_ticker_new(void *inner,
                                      uint64_t (*interval)(void*),
                                      void (*action)(void*, Context<Message>*),
                                      bool (*is_finished)(void*));

void behaviour_ticker_free(TickerBehaviour *ticker);

SequentialBehaviour *behaviour_sequential_new(void *inner,
                                              SequentialBehaviourQueue<Message> *initial_behaviours,
                                              void (*handle_child_message)(void*, Message*),
                                              void (*after_child_action)(void*, Context<Message>*));

void behaviour_sequential_free(SequentialBehaviour *sequential);

SequentialBehaviourQueue<Message> *behaviour_sequential_queue_new();

void behaviour_sequential_queue_add_behaviour_oneshot(SequentialBehaviourQueue<Message> *queue,
                                                      OneShotBehaviour *oneshot);

void behaviour_sequential_queue_add_behaviour_cyclic(SequentialBehaviourQueue<Message> *queue,
                                                     CyclicBehaviour *cyclic);

void behaviour_sequential_queue_add_behaviour_ticker(SequentialBehaviourQueue<Message> *queue,
                                                     TickerBehaviour *ticker);

void behaviour_sequential_queue_add_behaviour_sequential(SequentialBehaviourQueue<Message> *queue,
                                                         SequentialBehaviour *sequential);

void behaviour_sequential_queue_free(SequentialBehaviourQueue<Message> *queue);

/**
 * Initialize the libraries global logger.
 *
 * Values less or equal to 0 disable logging. Values from 1 to 5 (and up) set respectively the levels;
 * error, warn, info, debug, trace.
 */
void initialize_logging(char level);

}  // extern "C"

}  // namespace framework::__ffi

#endif  // FRAMEWORK_CORE_H
