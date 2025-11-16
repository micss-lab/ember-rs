#ifndef EMBER_CORE_H
#define EMBER_CORE_H

#include "inttypes.h"

namespace ember::__ffi {

template<typename S = void, typename E = void>
struct Agent;

template<typename E = void>
struct BehaviourVec;

struct Container;

template<typename E = void>
struct Context;

template<typename E = void>
struct CyclicBehaviour;

template<typename S = void, typename T = void, typename E = void>
struct Fsm;

template<typename E = void>
struct FsmBehaviour;

template<typename S = void, typename T = void, typename E = void>
struct FsmBuilder;

template<typename T = void, typename E = void>
struct FsmEvent;

struct Message;

struct MessageEnvelope;

struct MessageFilter;

template<typename E = void>
struct OneShotBehaviour;

template<typename E = void>
struct SequentialBehaviour;

template<typename E = void>
struct TickerBehaviour;

struct Event {
  void *inner;
};

struct AgentState {
  void *inner;
};

using BehaviourId = uint32_t;

struct ContainerPollResult {
  int32_t status;
  bool should_stop;
};

struct ContentView {
  const uint8_t *data;
  uintptr_t len;
};

extern "C" {

Event *event_new(void *event);

void event_free(Event *event);

AgentState *agent_state_new(void *agent_state);

void agent_state_free(AgentState *agent_state);

Agent<AgentState, Event> *agent_new(const char *name, AgentState *agent_state);

void agent_free(Agent<AgentState, Event> *agent);

void agent_add_behaviour_oneshot(Agent<AgentState, Event> *agent, OneShotBehaviour<Event> *oneshot);

void agent_add_behaviour_cyclic(Agent<AgentState, Event> *agent, CyclicBehaviour<Event> *cyclic);

void agent_add_behaviour_ticker(Agent<AgentState, Event> *agent, TickerBehaviour<Event> *ticker);

void agent_add_behaviour_sequential(Agent<AgentState, Event> *agent,
                                    SequentialBehaviour<Event> *sequential);

void agent_add_behaviour_fsm(Agent<AgentState, Event> *agent, FsmBehaviour<Event> *fsm);

OneShotBehaviour<Event> *behaviour_oneshot_new(void *inner, void (*action)(void*,
                                                                           Context<Event>*,
                                                                           AgentState*));

void behaviour_oneshot_free(OneShotBehaviour<Event> *oneshot);

CyclicBehaviour<Event> *behaviour_cyclic_new(void *inner,
                                             void (*action)(void*, Context<Event>*, AgentState*),
                                             bool (*is_finished)(void*));

void behaviour_cyclic_free(CyclicBehaviour<Event> *cyclic);

TickerBehaviour<Event> *behaviour_ticker_new(void *inner,
                                             uint64_t (*interval)(void*),
                                             void (*action)(void*, Context<Event>*, AgentState*),
                                             bool (*is_finished)(void*));

void behaviour_ticker_free(TickerBehaviour<Event> *ticker);

BehaviourVec<Event> *behaviour_vec_new();

void behaviour_vec_add_behaviour_oneshot(BehaviourVec<Event> *behaviour_vec,
                                         OneShotBehaviour<Event> *oneshot);

void behaviour_vec_add_behaviour_cyclic(BehaviourVec<Event> *behaviour_vec,
                                        CyclicBehaviour<Event> *cyclic);

void behaviour_vec_add_behaviour_ticker(BehaviourVec<Event> *behaviour_vec,
                                        TickerBehaviour<Event> *ticker);

void behaviour_vec_add_behaviour_sequential(BehaviourVec<Event> *behaviour_vec,
                                            SequentialBehaviour<Event> *sequential);

void behaviour_vec_add_behaviour_fsm(BehaviourVec<Event> *behaviour_vec, FsmBehaviour<Event> *fsm);

void behaviour_vec_free(BehaviourVec<Event> *behaviour_vec);

SequentialBehaviour<Event> *behaviour_sequential_new(void *inner,
                                                     BehaviourVec<Event> *initial_behaviours,
                                                     void (*handle_child_event)(void*, Event*),
                                                     void (*after_child_action)(void*,
                                                                                Context<Event>*,
                                                                                AgentState*));

void behaviour_sequential_free(SequentialBehaviour<Event> *sequential);

FsmBehaviour<Event> *behaviour_fsm_behaviour_new(void *inner,
                                                 Fsm<AgentState, const char*, Event> *fsm,
                                                 void (*handle_child_event)(void*, Event*),
                                                 void (*after_child_action)(void*,
                                                                            Context<Event>*,
                                                                            AgentState*));

void behaviour_fsm_behaviour_free(FsmBehaviour<Event> *fsm);

FsmBuilder<AgentState, const char*, Event> *behaviour_fsm_builder_new();

BehaviourId behaviour_fsm_builder_add_behaviour_oneshot(FsmBuilder<AgentState, const char*, Event> *builder,
                                                        OneShotBehaviour<FsmEvent<const char*, Event>> *oneshot,
                                                        bool is_final);

BehaviourId behaviour_fsm_builder_add_behaviour_cyclic(FsmBuilder<AgentState, const char*, Event> *builder,
                                                       CyclicBehaviour<FsmEvent<const char*, Event>> *cyclic,
                                                       bool is_final);

BehaviourId behaviour_fsm_builder_add_behaviour_ticker(FsmBuilder<AgentState, const char*, Event> *builder,
                                                       TickerBehaviour<FsmEvent<const char*, Event>> *ticker,
                                                       bool is_final);

BehaviourId behaviour_fsm_builder_add_behaviour_sequential(FsmBuilder<AgentState, const char*, Event> *builder,
                                                           SequentialBehaviour<FsmEvent<const char*, Event>> *sequential,
                                                           bool is_final);

BehaviourId behaviour_fsm_builder_add_behaviour_fsm(FsmBuilder<AgentState, const char*, Event> *builder,
                                                    FsmBehaviour<FsmEvent<const char*, Event>> *fsm,
                                                    bool is_final);

void behaviour_fsm_builder_add_default_transition(FsmBuilder<AgentState, const char*, Event> *builder,
                                                  BehaviourId src,
                                                  BehaviourId dest);

void behaviour_fsm_builder_add_transition(FsmBuilder<AgentState, const char*, Event> *builder,
                                          BehaviourId src,
                                          BehaviourId dest,
                                          const char *trigger);

Fsm<AgentState, const char*, Event> *behaviour_fsm_builder_build(FsmBuilder<AgentState, const char*, Event> *builder,
                                                                 BehaviourId start_behaviour);

FsmEvent<const char*, Event> *fsm_event_transition_new(const char *transition);

FsmEvent<const char*, Event> *fsm_event_event_new(Event *event);

void fsm_event_free(FsmEvent<const char*, Event> *event);

void context_emit_fsm_event(Context<FsmEvent<const char*, Event>> *context,
                            FsmEvent<const char*, Event> *event);

OneShotBehaviour<FsmEvent<const char*, Event>> *behaviour_fsm_child_behaviour_oneshot_new(void *inner,
                                                                                          void (*action)(void*,
                                                                                                         Context<FsmEvent<const char*, Event>>*,
                                                                                                         AgentState*));

void behaviour_fsm_child_behaviour_oneshot_free(OneShotBehaviour<FsmEvent<const char*, Event>> *oneshot);

CyclicBehaviour<FsmEvent<const char*, Event>> *behaviour_fsm_child_behaviour_cyclic_new(void *inner,
                                                                                        void (*action)(void*,
                                                                                                       Context<FsmEvent<const char*, Event>>*,
                                                                                                       AgentState*),
                                                                                        bool (*is_finished)(void*));

void behaviour_fsm_child_behaviour_cyclic_free(CyclicBehaviour<FsmEvent<const char*, Event>> *cyclic);

TickerBehaviour<FsmEvent<const char*, Event>> *behaviour_fsm_child_behaviour_ticker_new(void *inner,
                                                                                        uint64_t (*interval)(void*),
                                                                                        void (*action)(void*,
                                                                                                       Context<FsmEvent<const char*, Event>>*,
                                                                                                       AgentState*),
                                                                                        bool (*is_finished)(void*));

void behaviour_fsm_child_behaviour_ticker_free(TickerBehaviour<FsmEvent<const char*, Event>> *ticker);

SequentialBehaviour<FsmEvent<const char*, Event>> *behaviour_fsm_child_behaviour_sequential_new(void *inner,
                                                                                                BehaviourVec<Event> *initial_behaviours,
                                                                                                void (*handle_child_event)(void*,
                                                                                                                           Event*),
                                                                                                void (*after_child_action)(void*,
                                                                                                                           Context<FsmEvent<const char*, Event>>*,
                                                                                                                           AgentState*));

void behaviour_fsm_child_behaviour_sequential_free(SequentialBehaviour<FsmEvent<const char*, Event>> *sequential);

FsmBehaviour<FsmEvent<const char*, Event>> *behaviour_fsm_child_behaviour_fsm_new(void *inner,
                                                                                  Fsm<AgentState, const char*, Event> *fsm,
                                                                                  void (*handle_child_event)(void*,
                                                                                                             Event*),
                                                                                  void (*after_child_action)(void*,
                                                                                                             Context<FsmEvent<const char*, Event>>*,
                                                                                                             AgentState*));

void behaviour_fsm_child_behaviour_fsm_free(FsmBehaviour<FsmEvent<const char*, Event>> *fsm);

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

void context_send_message(Context<Event> *context, MessageEnvelope *message);

Message *context_receive_message(Context<Event> *context);

Message *context_receive_message_with_filter(Context<Event> *context, MessageFilter *filter);

Message *message_new(char performative,
                     const char *const *receivers,
                     uintptr_t receivers_len,
                     const char *ontology,
                     uint8_t *content,
                     uintptr_t content_len);

void message_free(Message *message);

MessageEnvelope *message_wrap_with_envelope(Message *message);

ContentView message_get_content(Message *message);

void message_filter_free(MessageFilter *filter);

MessageFilter *message_filter_all();

MessageFilter *message_filter_none();

MessageFilter *message_filter_performative(char performative);

MessageFilter *message_filter_ontology(const char *ontology);

/**
 * Initialize the libraries global logger.
 *
 * Values less or equal to 0 disable logging. Values from 1 to 5 (and up) set respectively the levels;
 * error, warn, info, debug, trace.
 */
void initialize_logging(char level);

extern uint8_t *malloc(uintptr_t size);

extern void free(uint8_t *ptr);

extern uint8_t *realloc(uint8_t *ptr, uintptr_t size);

}  // extern "C"

}  // namespace ember::__ffi

#endif  // EMBER_CORE_H
