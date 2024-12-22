#ifndef FRAMEWORK_CORE_H
#define FRAMEWORK_CORE_H

#include "inttypes.h"

namespace framework::__ffi {

template<typename M = void>
struct Agent;

struct Container;

template<typename M = void>
struct Context;

struct Message;

struct OneShotBehaviour;

extern "C" {

void initialize_allocator();

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

Agent<Message> *agent_new(const char *name);

void agent_free(Agent<Message> *agent);

void agent_add_behaviour_oneshot(Agent<Message> *agent, OneShotBehaviour *oneshot);

OneShotBehaviour *behaviour_oneshot_new(void *inner, void (*action)(void*, Context<Message>*));

void behaviour_oneshot_free(OneShotBehaviour *oneshot);

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
