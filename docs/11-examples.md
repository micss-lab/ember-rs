# 11. Examples

The [`examples/`](../examples) crate contains runnable programs, each demonstrating one feature. This
page is a guided tour. Run any of them on the host with:

```sh
cargo run-local --bin <name>
```

Or flash to an ESP32 with `cargo run-esp --bin <name>`. All examples share the `setup_example!()`
macro, which generates the correct `main` for host and device and initialises `TRACE`-level logging.

## Reactive-agent examples

### `behaviour_cyclic`
The simplest agent: a single `CyclicBehaviour` that runs on every tick. Start here.

### `behaviour_oneshot`
Two `OneShotBehaviour`s that each run once; one stops the platform when done. Shows one-shot lifecycle
and `ctx.stop_platform()`.

### `behaviour_fsm`
A manager and a worker coordinating via an FSM behaviour and internal `FsmEvent`s (task sent →
acknowledged → finished). Demonstrates complex behaviours and event-driven state transitions.

### `block_behaviour`
A cyclic receiver that calls `ctx.block_behaviour()` when no message is available, so it sleeps
instead of spinning until a message arrives. The key pattern for message-driven reactive agents.

### `remove_behaviour`
An agent with two behaviours where one removes the other (by `BehaviourId`) after a number of
iterations. Shows dynamic, runtime modification of an agent's behaviour set.

### `remove_agent`
An agent that self-terminates via `ctx.remove_agent()` after handling a fixed number of messages,
triggering AMS deregistration. (Support for this is best-effort.)

### `lifetime`
An agent whose behaviour holds borrowed, lifetime-bearing state (`&'a str`). Demonstrates that agents
can carry non-`'static` data.

### `client_server`
Two agents in one container: a `TickerBehaviour` client sends `Metrics` every 5 s; a `CyclicBehaviour`
server receives and logs them, blocking when idle. A complete intra-container messaging example,
including a custom `Content::Other` payload and `From`/`FromStr` conversions.

### `cross_platform_client_server`
The `client_server` scenario split across platform boundaries, exercising the MTS and a communication
channel so client and server can live on different containers/devices.

### `esp_now_client_server`
The same client/server pattern over the **ESP-NOW** channel between two ESP32 devices. Build for the
ESP32 target and flash to hardware (or run under Wokwi).

### `sensors`
A broader demo used to present project progress: a one-shot init behaviour, a periodic alive
broadcast, and sequential behaviours. A good tour of mixing behaviour kinds on one agent.

### `traffic`
A traffic light modelled as an `Fsm` behaviour cycling red → green → orange, with each light a child
behaviour and a `Switch` trigger driving transitions.

## BDI-agent examples

### `bdi_coffee`
The classic "coffee maker" BDI agent built with the **Rust API directly** (no macro): manually
constructed `KnowledgeBase`, `PlanLibrary`, `Plan`s, and a hand-written `FromTerm`. Read this to see
what `#[bdi_agent]` generates under the hood.

### `bdi_coffee_asl`
The **same coffee agent written in the AgentSpeak DSL** via `#[bdi_agent(asl = { … })]`, with
user-defined actions (`#[bdi_actions]`), custom term types (`FromTerm`, `#[ember(transparent)]`), and
a `Thermometer` sensor producing `SensorReading` percepts. The canonical BDI example: compare it side
by side with `bdi_coffee`.

### `bdi_rules`
A reactor-monitoring agent driven almost entirely by **rules** (derived beliefs): a chain of `:-`
clauses infers `danger_level_high`, `needs_evacuation`, `initiate_scram`, etc. from raw facts.
Demonstrates logic-programming-style reasoning, including disjunction via repeated heads and `not`.

### `bdi_rule_body_vars`
Rules with **variables in the body** (`system_critical :- part(Comp, broken)`) that quantify over the
belief base. Shows how a single rule reacts to any matching fact.

### `bdi_logistics`
A delivery robot that plans a **multi-step route** through `connected/2` facts using recursive
`!go_to` subgoals, updating its `at/2` belief as it moves. Demonstrates subgoal composition and
recursive planning.

### `bdi_smart_home`
A **context-sensitive** automation agent: the same `+presence(Room)` event triggers different plans
depending on `time(night)` vs `time(day)`. Shows plan context guards and belief-triggered plans.

### `bdi_send`
Two BDI agents sharing beliefs over the **`ember-bdil`** language: a sender `.send`s
`resource(water)`; the receiver reacts to the resulting `message(resource(X))` belief. The runnable
version of [BDI §7.13](./07-bdi-agents.md#713-inter-agent-belief-sharing).

## Where to look next

- Reactive API details → [Reactive Agents](./06-reactive-agents.md)
- BDI language reference → [BDI Agents](./07-bdi-agents.md)
- Running on hardware → [Embedded & ESP32](./10-embedded-esp32.md)
