# 5. Messaging

Messages are how Ember agents coordinate. Ember models messages on the **FIPA Agent Communication
Language (ACL)**, with a pluggable content model and two on-the-wire representations. This page
covers the message types you use from agent code and the machinery underneath them.

## 5.1 The `Message` type

A message is a [`Message`](../ember-core/src/message.rs):

```rust
pub struct Message {
    pub performative: Performative,
    pub receiver: Option<Receiver>,
    pub ontology: Option<String>,
    pub other: Option<BTreeMap<String, BString>>,
    pub content: Option<Content>,
}
```

- **`performative`**: the communicative intent (see below).
- **`receiver`**: one agent (`Receiver::Single(Aid)`) or several (`Receiver::Multiple(set)`).
- **`ontology`**: the vocabulary the content is expressed in (e.g. the agent-management ontology).
- **`other`**: arbitrary user-defined key/value parameters.
- **`content`**: the payload; its content *language* is encoded in the `Content` variant itself
  (there is no separate `language` field).

> Ember implements the most-used FIPA ACL parameters. Others from the spec (`sender`, `reply-to`,
> `conversation-id`, `protocol`, `reply-by`, …) are stubbed out and on the roadmap.

## 5.2 Performatives

The [`Performative`](../ember-core/src/message.rs) enum covers the full FIPA set:
`accept-proposal`, `agree`, `cancel`, `cfp`, `confirm`, `disconfirm`, `failure`, `inform`,
`inform-if`, `inform-ref`, `not-understood`, `propose`, `query-if`, `query-ref`, `refuse`,
`reject-proposal`, `request`, `request-when`, `request-whenever`, `subscribe`, `proxy`, `propagate`.

Performatives round-trip to their canonical FIPA strings (`Performative::from_str`, `Display`).

## 5.3 Agent identifiers (`Aid`)

An [`Aid`](../ember-core/src/agent/aid.rs) is a FIPA agent identifier of the form `name@host`. The
most common constructor is `Aid::local("name")`, producing `name@local`: an agent on the current
container. Remote agents carry a real host component, and the MTS uses it to decide whether to deliver
locally or over a communication channel.

```rust
use ember::agent::Aid;

let a = Aid::local("server");          // server@local
let b: Aid = "worker@device-2".parse().unwrap();
```

## 5.4 Content and content languages

Message content is a [`Content`](../ember-core/src/message/content.rs) enum. Each variant *is* a
content language:

| Variant                   | `.language()` | Meaning                                                             |
| ------------------------- | ------------- | ------------------------------------------------------------------- |
| `Content::FipaSl0(_)`     | `fipa-sl0`    | FIPA Semantic Language, profile 0: used by the management ontology |
| `Content::Bdil(_)`        | `ember-bdil`  | The BDI belief-sharing language (see [`ember-bdil`](./08-bdil.md))  |
| `Content::Bytes(_)`       | `bytes`       | Opaque bytes that are (de)coded in transit                          |
| `Content::Other { … }`    | user string   | Any other language, carried as raw bytes with a language tag        |

The content language is therefore *implicit in the type*: you never set a language string by hand for
the built-in languages. For an ad-hoc payload, use `Content::Other`:

```rust
use ember::message::Content;

let content = Content::Other {
    language: None,
    content: format!("{temp},{humidity}").into(),
};
```

Unknown incoming languages are preserved rather than rejected: they are stored as
`Content::Other { language, content }` so nothing is lost, even if the receiver cannot interpret them.

## 5.5 Sending and receiving

Agents never touch the MTS directly. They interact with their **environment/context**, which both
reactive and BDI contexts deref to:

```rust
// send
ctx.send_message(Message {
    performative: Performative::Inform,
    receiver: Some(Aid::local("server").into()),
    ontology: None,
    other: None,
    content: Some(/* … */),
});

// receive (optionally filtered): returns None if nothing matches
if let Some(msg) = ctx.receive_message(/* filter: */ None) {
    // handle it
}
```

`receive_message` takes a message *and removes it* from the inbox. Any messages left in the inbox at
the end of a tick are returned to the directory and offered again next tick.

## 5.6 Message filters

`receive_message` accepts an optional [`MessageFilter`](../ember-core/src/message/filter.rs) so an
agent (or a specific behaviour) can pull only the messages it cares about:

```rust
use ember::message::MessageFilter;

// only informs
let f = MessageFilter::performative(Performative::Inform);

// informs in the ember-bdil language, but not from the management ontology
let f = MessageFilter::and([
    MessageFilter::performative(Performative::Inform),
    MessageFilter::language("ember-bdil"),
    MessageFilter::ontology("fipa-agent-management").negated(),
]);
```

Filters compose: `all()`, `none()`, `performative(_)`, `ontology(_)`, `language(_)`, combined with
`and([...])`, `or([...])`, and `.negated()`. `matches(&message)` evaluates one against a message.

## 5.7 Wire representations

When a message leaves the container over a communication channel it is serialised. Ember implements
two FIPA ACL representations, chosen per envelope via `AclRepresentation`:

- **Bit-efficient** (`ember-core/src/message/repr/.../bit_efficient/`): a compact binary encoding of
  envelopes and payloads following the FIPA bit-efficient philosophy: every known keyword is a single
  predefined byte.
- **String** (`ember-core/src/message/repr/payload/string/`): the human-readable FIPA string form,
  useful for debugging and interoperability. `Message` implements `Display` using this encoding, so
  `format!("{message}")` gives you a readable dump.

A transport message is wrapped in a stack of **envelopes** (`MessageEnvelope`: `to`, `from`, `date`,
`acl_representation`, and extension parameters). Envelopes let intermediate transports annotate a
message without touching its payload: the payload can even remain opaque `Bytes` if a hop cannot
decode it.

## 5.8 Next

- [Communication Channels](./09-communication-channels.md): how serialised messages cross devices.
- [The `ember-bdil` Content Language](./08-bdil.md): the belief-sharing content language in detail.
