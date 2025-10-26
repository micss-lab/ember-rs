#ifndef EMBER_MESSAGE_FILTER_H
#define EMBER_MESSAGE_FILTER_H

#include "Message.h"

#include "../EmberCore.h"

namespace ember {

namespace message {

class MessageFilter: 
    public Object<__ffi::MessageFilter> {
  private:
    MessageFilter(__ffi::MessageFilter*);

  public:
    static MessageFilter all();
    static MessageFilter none();
    static MessageFilter performative(Performative performative);
    static MessageFilter ontology(const char* ontology);
};

} // namespace message

} // namespace ember

#endif // EMBER_MESSAGE_FILTER_H
