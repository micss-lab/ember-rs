#ifndef EMBER_H
#define EMBER_H

#include <inttypes.h>

#include "EmberCore.h"

#include "Agent.h"
#include "Container.h"
#include "Unit.h"

#include "behaviour/Event.h"
#include "behaviour/Context.h"
#include "behaviour/Behaviour.h"
#include "behaviour/OneShotBehaviour.h"
#include "behaviour/CyclicBehaviour.h"
#include "behaviour/TickerBehaviour.h"
#include "behaviour/SequentialBehaviour.h"
#include "behaviour/FsmBehaviour.h"


namespace ember {

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
 * Sets up the embers required resources such as the logger and the allocator.
 */
static void initialize(logging::LogLevel level) {
    if (INITIALIZED) {
        return;
    }
    __ffi::initialize_logging((int) level);
    INITIALIZED = true;
};

} // namespace ember

#endif // EMBER_H
