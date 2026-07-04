# 3. Architecture

Ember is a stack of small, single-responsibility crates. This page explains what each crate is
responsible for, how they depend on one another, and the key traits that stitch the layers together.

## 3.1 The crate stack

```
┌────────────────────────────────────────────────────────────────────────────────────────────────────────────────┐
│                                                     ember                                                      │ façade / runtime
│                      Container · message transport · agent management · public re-exports                      │
├───────────────────────────────────────────────────────────────────┬──────────────────────┬─────────────────────┤
│                      agent architectures                          │      ember-fipa      │     ember-acc       │ agent architectures
│      ember-reactive      ·               ember-bdi                │   AMS · ontologies   │ ESP-NOW·HTTP·custom │
│ (behaviour-based agents)   (belief-desire-intention-based agents) │                      │                     │ shared services
├───────────────────────────────────────────────────────────────────┴──────────────────────┴─────────────────────┤
│                                                   ember-core                                                   │
│                    Agent trait · Aid · Environment · FIPA ACL messages · content languages                     │ primitives
├──────────────────────────────────────────────────────────┬─────────────────────────────────────────────────────┤
│                        ember-time                        │                     ember-util                      │
│               tick sources · delay drivers               │          no_std sync · comparison helpers           │ platform / utils
└──────────────────────────────────────────────────────────┴─────────────────────────────────────────────────────┘
```

Dependencies point downward. Nothing in `ember-core` knows about containers, hardware, or a specific
agent architecture.

## 3.2 Crate responsibilities

### `ember-core`: primitives

The platform-agnostic foundation. It is deliberately small and defines the vocabulary every other
crate speaks:

- **[`Agent`](../ember-core/src/agent.rs)**: the entire contract the runtime needs:

  ```rust
  pub trait Agent {
      fn update(&mut self, environment: &mut Environment) -> bool;
      fn get_name(&self) -> Cow<str>;
  }
  ```

  `update` runs one tick and returns whether the agent has finished; `get_name` yields its local
  name. Both reactive and BDI agents implement this trait.

- **[`Aid`](../ember-core/src/agent/aid.rs)**: a FIPA agent identifier (`name@host`).
- **[`Environment`](../ember-core/src/environment.rs)**: the agent's I/O surface for a tick: an
  inbox (`message_inbox`), an outbox (`message_outbox`), and platform controls (`stop_platform`).
- **Messaging**: the [`Message`](../ember-core/src/message.rs) type, the pluggable
  [`Content`](../ember-core/src/message/content.rs) model, `MessageFilter`, and both the
  bit-efficient and string wire representations. Covered in [Messaging](./05-messaging.md).

### `ember-reactive`: behaviour-based agents

Implements the reactive agent architecture: `ReactiveAgent`, the `Behaviour` trait and its simple and
complex variants, the behaviour scheduler, and the behaviour `Context`. Covered in
[Reactive Agents](./06-reactive-agents.md).

### `ember-bdi`: BDI agents

Implements the Belief-Desire-Intention architecture: the belief base (with rules and unification),
the plan library, the event queue, the intention stack, built-in and user actions, and sensors. It
re-exports the proc macros from its `macros` sub-crate. Covered in [BDI Agents](./07-bdi-agents.md).

- **`ember-bdi/bdil`**: the standalone bit-efficient codec for the BDI content language
  (`ember-bdil`), used to share belief state between agents. Covered in [`ember-bdil`](./08-bdil.md).
- **`ember-bdi/macros`**: the proc-macro crate: the AgentSpeak parser and the code generation for
  `#[bdi_agent]`, `#[bdi_actions]`, and the `IntoLiteral` / `Percept` / `FromTerm` derives.

### `ember-fipa`: agent management

The FIPA management pieces shared by both agent kinds:

- The **AMS agent** (`ember-fipa/src/agent/ams.rs`), a privileged agent that owns the agent
  directory.
- The **agent-management ontology** (`ember-fipa/src/ontology.rs`): register/deregister actions and
  agent descriptions.
- **`FipaAgent`**: a reusable *component* (not a base class). Both `ReactiveAgent` and `BdiAgent`
  embed one and call its `update` first thing each tick; it registers the agent with the AMS on
  start-up and reports an execution state (`Initiated` → `Active`). More execution states might be 
  added in the future.

### `ember-acc`: communication channels

Agent Communication Channels: the transports the MTS uses to reach other containers/devices,
namely ESP-NOW, HTTP, and a `custom` escape hatch via the `Acc` trait. Selected with feature flags.
Covered in [Communication Channels](./09-communication-channels.md).

### `ember-time`: platform time

Tick sources and delay drivers, with `std` and `esp32` backends behind features. Used by
`TickerBehaviour` and anything time-sensitive.

### `ember-util`: utilities

Small `no_std` helpers: synchronisation primitives built on `critical-section` (e.g. an atomic
counter used to hand out behaviour IDs) and comparison utilities (e.g. total ordering for `f32`).

### `ember`: the facade

The top crate ties everything together:

- Hosts the **`Container`** and the **MTS** wiring.
- Exposes a curated public API through re-exports. The important namespaces are:
  - `ember::Container`
  - `ember::agent::{Agent, Aid}`
  - `ember::agent::reactive::{ReactiveAgent, behaviour}`
  - `ember::agent::bdi::{BdiAgent, bdi_agent, bdi_actions, …}`
  - `ember::message`, `ember::environment`
  - `ember::_crates::{core, reactive, bdi, fipa, acc}` for direct access to a sub-crate.
- Defines the **feature flags** users toggle:

  | Feature       | Effect                                                            |
  | ------------- | ---------------------------------------------------------------- |
  | `std`         | Enables host conveniences (`ember-time/std`)                     |
  | `esp32`       | ESP32 time driver (`ember-time/esp32`)                           |
  | `acc-espnow`  | ESP-NOW communication channel                                    |
  | `acc-http`    | HTTP communication channel (implies `std`)                       |
  | `acc-custom`  | Enables plugging in a custom `Acc`                               |

## 3.3 How a tick flows through the layers

Understanding one tick clarifies the whole architecture:

1. **`Container::poll`** pumps the MTS (delivering inbound transport messages into the local agent
   directory), lets privileged agents (the AMS) run, then iterates the agents.
2. For each agent, the container builds an **`Environment`** pre-loaded with that agent's inbox and
   calls **`Agent::update`**.
3. The agent's `update` first ticks its embedded **`FipaAgent`** component (AMS registration), then
   runs its architecture-specific logic: scheduling behaviours (reactive) or advancing the BDI
   reasoning cycle.
4. During `update` the agent reads from the inbox and pushes to the outbox via the `Environment`, and
   may set `stop_platform`.
5. Back in the container, queued outbound messages are handed to the **MTS**, unhandled inbox
   messages are returned to the directory, and the agent is rescheduled unless it reported that it is
   finished.

The container never inspects *what kind* of agent it is running: it only calls `update`. That single
seam is what makes reactive and BDI agents interchangeable and new architectures cheap to add.

## 3.4 Next

- [The Container](./04-container.md): the runtime loop in full detail.
- [Messaging](./05-messaging.md): the data that flows between agents.
