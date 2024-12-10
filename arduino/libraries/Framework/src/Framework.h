#ifndef FRAMEWORK_H
#define FRAMEWORK_H

#include <inttypes.h>
#include <functional>

#include "FrameworkCore.h"

#include "Agent.h"
#include "Container.h"
#include "Behaviour.h"

namespace framework {

namespace logging {
    enum class LogLevel {
        Off = 0,
        Error = 1,
        Warn = 2,
        Info = 3,
        Debug = 4,
        Trace = 5,
    };
}

static bool INITIALIZED = false;
/**
 * Sets up the frameworks required resources such as the logger and the allocator.
 */
static void initialize(logging::LogLevel level) {
    if (INITIALIZED) {
        return;
    }
    __ffi::initialize_logging((int) level);
    __ffi::initialize_allocator();
    INITIALIZED = true;
};

} // namespace framework

#endif // FRAMEWORK_H
