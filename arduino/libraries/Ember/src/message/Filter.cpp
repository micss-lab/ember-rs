#include "Filter.h"

using namespace ember::message;

MessageFilter::MessageFilter(__ffi::MessageFilter* inner):
    Object(inner, __ffi::message_filter_free) {}

MessageFilter MessageFilter::all() {
    return std::move(MessageFilter(__ffi::message_filter_all()));
}

MessageFilter MessageFilter::none() {
    return std::move(MessageFilter(__ffi::message_filter_none()));
}

MessageFilter MessageFilter::performative(Performative performative) {
    return std::move(MessageFilter(
        __ffi::message_filter_performative(static_cast<char>(performative))
    ));
}

MessageFilter MessageFilter::ontology(const char* ontology) {
    return std::move(MessageFilter(__ffi::message_filter_ontology(ontology)));
}
