#ifndef EMBER_CORE_H
#define EMBER_CORE_H

#include "inttypes.h"

namespace ember::__ffi {

template<typename S = void, typename E = void>
struct Agent;

struct BehaviourVec;

struct Container;

template<typename E = void>
struct Context;

struct CyclicBehaviour;

struct Message;

struct OneShotBehaviour;

struct SequentialBehaviour;

struct TickerBehaviour;

struct Event {
  void *inner;
};

struct AgentState {
  void *inner;
};

struct ContainerPollResult {
  int32_t status;
  bool should_stop;
};

extern "C" {

void initialize_allocator();

Event *event_new(void *event);

void event_free(Event *event);

AgentState *agent_state_new(void *agent_state);

void agent_state_free(AgentState *agent_state);

Agent<AgentState, Event> *agent_new(const char *name, AgentState *agent_state);

void agent_free(Agent<AgentState, Event> *agent);

void agent_add_behaviour_oneshot(Agent<AgentState, Event> *agent, OneShotBehaviour *oneshot);

void agent_add_behaviour_cyclic(Agent<AgentState, Event> *agent, CyclicBehaviour *cyclic);

void agent_add_behaviour_ticker(Agent<AgentState, Event> *agent, TickerBehaviour *ticker);

void agent_add_behaviour_sequential(Agent<AgentState, Event> *agent,
                                    SequentialBehaviour *sequential);

OneShotBehaviour *behaviour_oneshot_new(void *inner, void (*action)(void*,
                                                                    Context<Event>*,
                                                                    AgentState*));

void behaviour_oneshot_free(OneShotBehaviour *oneshot);

CyclicBehaviour *behaviour_cyclic_new(void *inner,
                                      void (*action)(void*, Context<Event>*, AgentState*),
                                      bool (*is_finished)(void*));

void behaviour_cyclic_free(CyclicBehaviour *cyclic);

TickerBehaviour *behaviour_ticker_new(void *inner,
                                      uint64_t (*interval)(void*),
                                      void (*action)(void*, Context<Event>*, AgentState*),
                                      bool (*is_finished)(void*));

void behaviour_ticker_free(TickerBehaviour *ticker);

BehaviourVec *behaviour_vec_new();

void behaviour_vec_add_behaviour_oneshot(BehaviourVec *behaviour_vec, OneShotBehaviour *oneshot);

void behaviour_vec_add_behaviour_cyclic(BehaviourVec *behaviour_vec, CyclicBehaviour *cyclic);

void behaviour_vec_add_behaviour_ticker(BehaviourVec *behaviour_vec, TickerBehaviour *ticker);

void behaviour_vec_add_behaviour_sequential(BehaviourVec *behaviour_vec,
                                            SequentialBehaviour *sequential);

void behaviour_vec_free(BehaviourVec *behaviour_vec);

SequentialBehaviour *behaviour_sequential_new(void *inner,
                                              BehaviourVec *initial_behaviours,
                                              void (*handle_child_event)(void*, Event*),
                                              void (*after_child_action)(void*,
                                                                         Context<Event>*,
                                                                         AgentState*));

void behaviour_sequential_free(SequentialBehaviour *sequential);

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

void container_add_agent(Container *container, Agent<AgentState, Event> *agent);

int32_t container_start(Container *container);

ContainerPollResult container_poll(Container *container);

void context_emit_event(Context<Event> *context, Event *event);

void context_stop_container(Context<Event> *context);

void context_remove_agent(Context<Event> *context);

void context_block_behaviour(Context<Event> *context);

Message *message_new(char performative,
                     const char *const *receivers,
                     uintptr_t receivers_len,
                     const char *ontology,
                     uint8_t *content,
                     uintptr_t content_len);

void message_free(Message *message);

/**
 * Initialize the libraries global logger.
 *
 * Values less or equal to 0 disable logging. Values from 1 to 5 (and up) set respectively the levels;
 * error, warn, info, debug, trace.
 */
void initialize_logging(char level);

}  // extern "C"

}  // namespace ember::__ffi

#endif  // EMBER_CORE_H
