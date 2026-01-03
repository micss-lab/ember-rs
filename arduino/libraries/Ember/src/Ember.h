#ifndef EMBER_H
#define EMBER_H

#define EMBER_ENABLE_ACC_ESPNOW // Allow the user to enable this manually for release.

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

#include "message/Message.h"
#include "message/Filter.h"

#include "acc/Acc.h"
#ifdef EMBER_ENABLE_ACC_ESPNOW
#include "acc/EspNow.h"
#endif // EMBER_ENABLE_ACC_ESPNOW

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
