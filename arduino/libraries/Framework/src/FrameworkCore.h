#ifndef FRAMEWORK_CORE_H
#define FRAMEWORK_CORE_H

#include "inttypes.h"

namespace framework::__ffi {

struct Agent;

struct Container;

struct Context;

template<typename S = void, typename P = void>
struct CyclicBehaviour;

template<typename S = void>
struct OneShotBehaviour;

template<typename S = void, typename PS = void>
struct SequentialBehaviour;

struct SimpleState {
  void *value;
  bool finished;
};

struct State {
  void *root;
  State *parent;
};

extern "C" {

void initialize_allocator();

/// Creates a new container instance.
///
/// # Safety
///
/// The ownership of the instance is transferred to the caller. Make sure to free the memory
/// with the accompanying [`container_free`].
Container *container_new();

void container_free(Container *container);

void container_add_agent(Container *container, Agent *agent);

int32_t container_start(Container *container);

Agent *agent_new(const char *name);

void agent_free(Agent *agent);

void agent_add_behaviour_oneshot(Agent *agent, OneShotBehaviour<void> *oneshot);

void agent_add_behaviour_cyclic(Agent *agent, CyclicBehaviour<SimpleState, void> *cyclic);

OneShotBehaviour<State> *behaviour_oneshot_new(State (*action)(Context*, State));

OneShotBehaviour<void> *behaviour_oneshot_new_void(void (*action)(Context*));

void behaviour_oneshot_free(OneShotBehaviour<State> *oneshot);

void behaviour_oneshot_free_void(OneShotBehaviour<void> *oneshot);

CyclicBehaviour<SimpleState, State> *behaviour_cyclic_new(SimpleState state,
                                                          State (*action)(Context*,
                                                                          SimpleState*,
                                                                          State));

CyclicBehaviour<SimpleState, void> *behaviour_cyclic_new_void(SimpleState state,
                                                              void (*action)(Context*, SimpleState*));

void behaviour_cyclic_free(CyclicBehaviour<SimpleState, State> *cyclic);

void behaviour_cyclic_free_void(CyclicBehaviour<SimpleState, void> *cyclic);

SequentialBehaviour<void*, State> *behaviour_sequential_new(void *state);

SequentialBehaviour<void*, void> *behaviour_sequential_new_void(void *state);

void behaviour_sequential_free(SequentialBehaviour<void*, State> *sequential);

void behaviour_sequential_free_void(SequentialBehaviour<void*, void> *sequential);

/// Initialize the libraries global logger.
///
/// Values less or equal to 0 disable logging. Values from 1 to 5 (and up) set respectively the levels;
/// error, warn, info, debug, trace.
void initialize_logging(char level);

}  // extern "C"

}  // namespace framework::__ffi

#endif  // FRAMEWORK_CORE_H
