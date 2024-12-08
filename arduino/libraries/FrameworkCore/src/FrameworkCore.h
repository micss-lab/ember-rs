
namespace ffi {

struct Agent;

struct Container;

extern "C" {

/// Creates a new container instance.
///
/// # Safety
///
/// The ownership of the instance is transferred to the caller. Make sure to free the memory
/// with the accompanying [`container_free`].
Container *container_new();

void container_add_agent(Container *container, Agent *agent);

int32_t container_start(Container *container);

void container_free(Container *container);

Agent *agent_new(const char *name);

void agent_free(Agent *agent);

/// Initialize the libraries global logger.
///
/// Values less or equal to 0 disable logging. Values from 1 to 5 (and up) set respectively the levels;
/// error, warn, info, debug, trace.
void initialize_logging(char level);

}  // extern "C"

}  // namespace ffi
