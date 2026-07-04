# Glossary

Definitions of the terms used throughout the Ember documentation.

### ACC (Agent Communication Channel)
A transport that delivers messages between agents on different containers/devices (ESP-NOW, HTTP, or a
custom channel). Implemented in `ember-acc` via the `Acc` trait. See
[Communication Channels](./09-communication-channels.md).

### ACL (Agent Communication Language)
The FIPA message language Ember models its `Message` type on: a performative, receiver, ontology, and
content. See [Messaging](./05-messaging.md).

### Action
A leaf step in a BDI plan body. Either a **built-in action** (`.log`, `.send`, `.stop_platform`) or a
**user-defined action** (a Rust method on the agent, declared with `#[bdi_actions]`). See
[BDI Agents §7.8–7.9](./07-bdi-agents.md#78-built-in-actions).

### AID (`Aid`)
An Agent Identifier of the form `name@host` (e.g. `server@local`). Ember's `Aid` type. See
[Messaging §5.3](./05-messaging.md#53-agent-identifiers-aid).

### AMS (Agent Management System)
The privileged agent that owns a container's agent directory and manages registration/lifecycle.
Addressable as `ams@local`. See [The Container §4.3](./04-container.md#43-the-agent-management-system-ams).

### AgentSpeak
The logic-based agent programming language family (as in Jason) that Ember's BDI DSL is modelled on.

### Behaviour
The unit of logic scheduled by a **reactive** agent. Simple (`Cyclic`, `OneShot`, `Ticker`) or complex
(`Fsm`, `Parallel`, `Sequential`). See [Reactive Agents](./06-reactive-agents.md).

### Belief
A fact a BDI agent holds, stored as a ground literal in its **belief base**. May be stored or
**derived** (via a rule). See [BDI Agents §7.4–7.5](./07-bdi-agents.md#74-beliefs).

### BDI (Belief-Desire-Intention)
A symbolic agent architecture: the agent holds beliefs, adopts goals (desires), and commits to plans
(intentions). Ember's reasoning-based agent kind. See [BDI Agents](./07-bdi-agents.md).

### Container
Ember's runtime: it hosts agents, schedules them cooperatively, and routes their messages. See
[The Container](./04-container.md).

### Content language
The language a message's content is expressed in (`fipa-sl0`, `ember-bdil`, `bytes`, or a user tag),
encoded implicitly by the `Content` variant. See [Messaging §5.4](./05-messaging.md#54-content-and-content-languages).

### `ember-bdil`
The bit-efficient FIPA content language BDI agents use to share belief state. See
[The `ember-bdil` Content Language](./08-bdil.md).

### Environment
An agent's I/O surface for a single tick: an inbox, an outbox, and platform controls. Both the reactive
and BDI contexts deref to it. See [Architecture §3.2](./03-architecture.md#32-crate-responsibilities).

### Event (BDI)
A change (belief added/removed, goal posted) that a BDI agent reacts to by selecting a plan. Not to be
confused with a reactive behaviour's internal **Event** type.

### FIPA
The Foundation for Intelligent Physical Agents, whose standards (ACL, AMS, MTS, AID) Ember adapts. See
[Introduction §1.2](./01-introduction.md#12-fipa).

### FSM (Finite-State Machine)
A complex reactive behaviour where each state is a child behaviour and typed triggers drive
transitions. See [Reactive Agents §6.4](./06-reactive-agents.md#64-complex-behaviours).

### Intention
A plan a BDI agent has committed to and is executing, held on its intention stack. An agent with no
remaining intentions reports itself finished.

### Literal
A structured term `functor(arg, …)`, optionally negated with `~`. The building block of beliefs,
goals, and BDI messages.

### MTS (Message Transport Service)
The container subsystem that routes messages: locally through the directory, or remotely through a
channel. See [The Container §4.4](./04-container.md#44-the-message-transport-service-mts).

### Ontology
The vocabulary a message's content is interpreted against (e.g. the agent-management ontology used by
AMS registration).

### Percept / Perceptor
A **perceptor** is a sensor polled each tick; the **percept** it returns is folded into the BDI agent's
belief base. See [BDI Agents §7.11](./07-bdi-agents.md#711-sensors-and-percepts).

### Performative
The communicative intent of a message (`inform`, `request`, `query-if`, …). See
[Messaging §5.2](./05-messaging.md#52-performatives).

### Plan
A BDI recipe: a triggering event, an optional context guard, and a body of steps. See
[BDI Agents §7.6](./07-bdi-agents.md#76-goals-and-plans).

### Reactive agent
Ember's behaviour-based agent kind (JADE-inspired). See [Reactive Agents](./06-reactive-agents.md).

### Rule
A logic-programming clause (`head :- body`) that derives a belief on demand. See
[BDI Agents §7.5](./07-bdi-agents.md#75-rules-derived-beliefs).

### Tick
One invocation of an agent's `update` by the container. Agents do a bounded amount of work per tick,
then yield.

### Unification
The logic-programming operation of matching two literals by binding variables, used when selecting
plans and evaluating rules.
