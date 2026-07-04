# 4. The Container

The **`Container`** is Ember's runtime. It hosts agents, schedules them cooperatively, routes their
messages, and manages their lifecycle through the AMS. This page describes what the container does
and how to configure it.

## 4.1 Creating a container

```rust
use ember::Container;

let container = Container::new()            // or Container::default()
    .with_agent(/* … */)
    .with_agent(/* … */);

container.start().expect("container error");
```

`Container::new()` and `Container::default()` are equivalent. Agents are added with the builder-style
`with_agent` (chainable) or the imperative `add_agent`. The container is generic over two lifetimes so
it can hold agents and communication channels that borrow from their surroundings: in practice you
rarely name them.

## 4.2 The run loop

`start()` simply calls `poll()` in a loop until `poll()` signals that the platform should stop:

```rust
pub fn start(mut self) -> Result<(), Box<dyn Error>> {
    loop {
        if self.poll()? {
            break Ok(());
        }
    }
}
```

`poll()` executes exactly one round of the cooperative scheduler. Each round:

1. **Pumps the MTS inbound.** `mts.receive_messages` drains every communication channel, rewrites
   each incoming message's destination to the local address, and delivers it into the local agent
   directory.
2. **Runs privileged agents.** The AMS (see below) is polled first, so registrations and other
   management actions take effect before ordinary agents run.
3. **Ticks each ordinary agent once.** The container pops agents from a queue, and for each one:
   - builds an `Environment` pre-loaded with the agent's pending inbox,
   - calls `Agent::update(&mut environment)`,
   - forwards everything the agent placed in `environment.message_outbox` to the MTS,
   - returns any messages the agent left unread to the directory,
   - honours `environment.stop_platform` (returns "stop" immediately if set),
   - reschedules the agent unless `update` returned `true` (finished).

Because a single `poll()` ticks every currently-queued agent once, agents make progress in
round-robin fashion. Each agent is expected to do a **bounded** amount of work per tick and then
yield: this is what keeps the single-threaded platform responsive.

> **Finished agents.** When an agent's `update` returns `true`, it is not rescheduled. A reactive
> agent never finishes (it returns `false`); a BDI agent reports finished when it has no remaining
> intentions.

## 4.3 The Agent Management System (AMS)

The AMS is a **privileged agent**: it can modify the container directly, unlike ordinary agents which
only see their `Environment`. Every container starts with exactly one AMS, addressable as
`ams@local`.

Its job is to own the **local agent directory** (internally the *ADT*, an "agent directory table"):
the authoritative map from local agent name to that agent's inbox.

The registration handshake works like this:

1. On its first tick, every agent's embedded `FipaAgent` component sends a `request` message to
   `ams@local` carrying a *register* action (from the agent-management ontology) with the agent's
   AID, e.g. `greeter@local`.
2. The AMS processes that request during the privileged-agents phase and inserts the agent into the
   directory.
3. Only once an agent is in the directory can messages be delivered to it. A message addressed to an
   unregistered local name is dropped with an error log.

Deregistration (agent self-termination) follows the same pattern in reverse; see the `remove_agent`
example.

## 4.4 The Message Transport Service (MTS)

The MTS routes messages. When an agent sends a message (via its `Environment`), the container hands
the resulting *transport message* to `mts.send_message`, which for each receiver:

- **Delivers locally** if the receiver resolves to a local agent: the message is pushed directly
  onto that agent's inbox in the directory. Proxy entries are followed transitively (with loop
  detection) so one local name can stand in for another.
- **Delivers remotely** otherwise: the message is handed to the configured communication
  **channels** (`ember-acc`), which serialise and transmit it to another device/container.

Inbound remote messages are picked up by `mts.receive_messages` at the top of each `poll()` and
delivered exactly as if they had originated locally.

The set of transports the MTS uses is configured on the container. Each requires the corresponding
feature flag:

```rust
// HTTP over Wi-Fi (requires feature `acc-http`, implies `std`)
let container = Container::new().with_http(8080);

// ESP-NOW between ESP32 devices (requires feature `acc-espnow`)
let container = Container::new().with_espnow(Some(sender), Some(receiver));

// A custom channel (requires feature `acc-custom`)
let container = Container::new().with_custom_acc(Box::new(my_channel));
```

See [Communication Channels](./09-communication-channels.md) for details on each transport.

## 4.5 Agent proxies

You can register a *proxy* so that a local name forwards to another AID:

```rust
container.with_agent_proxy("printer", Aid::general("real-printer", "some-other-platform"));
```

Messages addressed to `printer@local` are then transparently re-routed to `real-printer@some-other-platform`. The
MTS resolves proxy chains and rejects loops.

## 4.6 Stopping the platform

Any agent can stop the whole container by calling `stop_platform()` on its environment/context. On
the next return from `update`, the container observes `stop_platform` and `poll()` returns `true`,
ending `start()`. BDI agents expose this as the `.stop_platform()` built-in action.

## 4.7 Next

- [Messaging](./05-messaging.md): the structure of the messages the MTS carries.
- [Reactive Agents](./06-reactive-agents.md) / [BDI Agents](./07-bdi-agents.md): what runs inside a tick.
