# `ember` - The embedded-first multi-agent platform

**Ember** is a multi-agent framework for resource-constrained embedded devices, written in Rust.
It provides the infrastructure for deploying communicating agents on microcontrollers, with the
ESP32 as the primary target. The design follows a layered, `no_std`-first architecture, separating
platform-agnostic abstractions from device-specific implementations, and keeping the agent
execution model decoupled from the agent implementation.

The framework draws on concepts from the [FIPA](http://www.fipa.org/) agent standards, including
the Agent Management System (AMS), the Message Transport Service (MTS), and the FIPA ACL message
model, adapted for the constraints of embedded environments. On top of the shared infrastructure it
ships two built-in agent architectures: **reactive** (JADE-inspired, behaviour-based) agents and
**BDI** (Belief-Desire-Intention) agents programmed in an AgentSpeak-like language.

> This project is research software developed as part of a master's thesis. APIs are unstable and
> subject to change.

**Full documentation** lives in the [`docs/`](./docs) directory (start at
[`docs/README.md`](./docs/README.md)). Contributors should read [`CONTRIBUTING.md`](./CONTRIBUTING.md).

---

## Architecture

Ember is organised into a stack of small, single-responsibility crates. Platform-agnostic
primitives live at the bottom, the two agent architectures sit in the middle as optional libraries,
and the `ember` façade crate assembles everything into a runnable platform.

```
┌────────────────────────────────────────────────────────────────────────────────────────────────────────────────┐
│                                                     ember                                                      │
│                      Container · message transport · agent management · public re-exports                      │
├───────────────────────────────────────────────────────────────────┬──────────────────────┬─────────────────────┤
│                      agent architectures                          │      ember-fipa      │     ember-acc       │
│      ember-reactive      ·               ember-bdi                │   AMS · ontologies   │ ESP-NOW·HTTP·custom │
│ (behaviour-based agents)   (belief-desire-intention-based agents) │                      │                     │
├───────────────────────────────────────────────────────────────────┴──────────────────────┴─────────────────────┤
│                                                   ember-core                                                   │
│                    Agent trait · Aid · Environment · FIPA ACL messages · content languages                     │
├──────────────────────────────────────────────────────────┬─────────────────────────────────────────────────────┤
│                        ember-time                        │                     ember-util                      │
│               tick sources · delay drivers               │          no_std sync · comparison helpers           │
└──────────────────────────────────────────────────────────┴─────────────────────────────────────────────────────┘
```

The codebase is split into the following crates:

### `ember-core`

The platform-agnostic foundation. Defines:

- **Agent trait & entity**: the minimal [`Agent`] trait (`update` + `get_name`) and the agent
  identifier ([`Aid`], a FIPA-style `name@host` address).
- **Environment**: the agent's window to the outside world: an inbox, an outbox, and platform
  controls (e.g. stopping the platform).
- **FIPA ACL messages**: the [`Message`] type (performative, receiver, ontology, content), the
  [`Content`] model with pluggable **content languages**.
- **Message filtering**: declarative filters used by agents to pull only the messages they care
  about out of the inbox.

### `ember-reactive`

The reactive, behaviour-based agent architecture, following a JADE-inspired model: an agent is an
object that schedules and executes a set of **behaviours**, with message passing as the primary
coordination mechanism.

- *Simple behaviours*: `CyclicBehaviour`, `OneShotBehaviour`, `TickerBehaviour`.
- *Complex behaviours*: `Fsm` (finite-state machine), `ParallelBehaviour`, `SequentialBehaviour`,
  plus behaviour blocking.
- A behaviour **context** for sending/receiving messages, emitting events, and dynamically adding,
  blocking, resetting, or removing behaviours at runtime.

### `ember-bdi`

The BDI (Belief-Desire-Intention) agent architecture. Agents are described declaratively in an
AgentSpeak/Jason-inspired language embedded directly in Rust via the `#[bdi_agent(asl = { … })]`
attribute macro. Provides:

- A **belief base** with logic-programming-style **rules** (derived beliefs) and unification.
- A **plan library**, an **event queue**, and an **intention stack** implementing the BDI
  reasoning cycle.
- **Built-in actions** (`.log`, `.send`, `.stop_platform`, …) and user-defined actions declared with
  `#[bdi_actions]`.
- **Sensors / perceptors** that turn environment readings into beliefs, with `#[derive(Percept)]`.
- Derive macros (`IntoLiteral`, `FromTerm`) for moving data between Rust types and BDI terms.
- **`ember-bdi/bdil`**: the *bit-efficient* codec for the BDI content language used to share belief
  state between agents over FIPA ACL (see [`spec/ember-bdil.md`](./spec/ember-bdil.md)).
- **`ember-bdi/macros`**: the proc-macro crate implementing the AgentSpeak parser and code
  generation.

### `ember-fipa`

FIPA agent-management building blocks shared by both agent kinds: the **AMS** agent, the
agent-management **ontology** (register/deregister actions, agent descriptions), and the reusable
`FipaAgent` component that registers an agent with the AMS on start-up.

### `ember-acc`

Agent Communication Channel implementations, the transports the MTS uses to reach agents on other
containers/devices:

- **ESP-NOW**: low-latency peer-to-peer communication between ESP32 devices, with a custom
  serialisation format.
- **HTTP**: communication over Wi-Fi networks (currently only on `std`-enabled targets).
- **Custom**: bring-your-own channel through the `Acc` trait.

### `ember-time`

Platform-specific time drivers. Provides the tick sources and delay abstractions used by
`TickerBehaviour` and other time-sensitive components, with backends for `std` hosts and the ESP32.

### `ember-util`

Small shared utilities: `no_std` synchronisation primitives (e.g. atomics behind
`critical-section`) and comparison helpers (e.g. total ordering for `f32`).

---

## Agent Kinds

ember is designed to support multiple agent architectures as separate, optional libraries. Each kind
runs with the same underlying container, messaging, and communication infrastructure.

| Crate            | Kind                          | Programmed with                                    |
| ---------------- | ----------------------------- | -------------------------------------------------- |
| `ember-reactive` | Reactive (behaviour-based)    | Rust traits (`CyclicBehaviour`, `Fsm`, …)          |
| `ember-bdi`      | BDI (Belief-Desire-Intention) | AgentSpeak-like DSL via `#[bdi_agent]` + Rust actions |

### Reactive (behaviour-based) agents

Reactive agents follow a JADE-inspired model: agents schedule a set of behaviours that run on each
tick of the container. Behaviours range from simple (`Cyclic`, `OneShot`, `Ticker`) to complex
compositions (`Fsm`, `Parallel`, `Sequential`).

```rust
struct Blink;

impl CyclicBehaviour for Blink {
    type AgentState = ();
    type Event = ();

    fn action(&mut self, ctx: &mut Context<Self::Event>, _: &mut Self::AgentState) {
        log::info!("tick");
    }

    fn is_finished(&self) -> bool { false }
}

Container::new()
    .with_agent(ReactiveAgent::new("blinker", ()).with_behaviour(Blink))
    .start()
    .unwrap();
```

### BDI agents

BDI agents are written declaratively. Beliefs, goals, and plans are expressed in an AgentSpeak-like
language; built-in and user actions bridge back into Rust.

```rust
#[derive(FromTerm)]
#[ember(transparent)]
struct Item(String);

#[bdi_agent(asl = {
    at(agent, home).
    !make_coffee.

    +!make_coffee : at(agent, kitchen) & have(coffee_beans)
      <- .log("info", "Enjoying a fresh cup of coffee!");
         .stop_platform().

    +!make_coffee : at(agent, kitchen)
      <- buy(coffee_beans);
         +have(coffee_beans);
         !make_coffee.

    +!make_coffee
      <- !go_to(kitchen);
         !make_coffee.

    +!go_to(Dest)
      <- -at(agent, home);
         +at(agent, Dest).
})]
struct CoffeeAgent;

#[bdi_actions]
impl CoffeeAgent {
    // a user-defined action, invoked by the `buy(coffee_beans)` step above
    fn buy(&mut self, item: Item) {
        log::info!("Buying {}", item.0);
    }
}

Container::new()
    .with_agent(CoffeeAgent.into_agent())
    .start()
    .unwrap();
```

See [`docs/07-bdi-agents.md`](./docs/07-bdi-agents.md) for the full language reference.

---

## Getting Started

### Prerequisites

- Rust `1.88.0` (pinned in [`rust-toolchain.toml`](./rust-toolchain.toml)).
- For ESP32 targets: the `xtensa` toolchain (installed via [`espup`](https://github.com/esp-rs/espup))
  and [`espflash`](https://github.com/esp-rs/espflash).
- Optional: [Nix](https://nixos.org/). A `flake.nix` provides a fully configured development shell
  (`nix develop`).

> **Note:** the default build target is `xtensa-esp32-none-elf` (see `.cargo/config.toml`). To build
> and run on your host machine, use the `-local` cargo aliases, which target
> `x86_64-unknown-linux-gnu`.

### Building

```sh
# Host build (all crates)
cargo build-local

# ESP32 build (requires the xtensa toolchain)
cargo build-esp
```

### Running the examples

Examples live in `examples/src/bin/` and run on the host with the `run-local` alias:

```sh
cargo run-local --bin behaviour_cyclic
cargo run-local --bin client_server
cargo run-local --bin bdi_coffee_asl
```

To flash an example onto an ESP32 (with the board connected):

```sh
cargo run-esp --bin sensors
```

For ESP32/Wokwi simulation see [`examples/wokwi.toml`](./examples/wokwi.toml) and
[`docs/10-embedded-esp32.md`](./docs/10-embedded-esp32.md).

### Examples

The `examples/` directory contains programs that demonstrate individual features:

| Example                          | Kind     | Description                                                        |
| -------------------------------- | -------- | ----------------------------------------------------------------- |
| `behaviour_cyclic`               | Reactive | A simple agent with a repeating behaviour                         |
| `behaviour_oneshot`              | Reactive | A behaviour that executes once and terminates                    |
| `behaviour_fsm`                  | Reactive | Finite-state-machine behaviour with two coordinating agents      |
| `block_behaviour`                | Reactive | Blocking a behaviour while waiting for a message                 |
| `remove_behaviour`               | Reactive | Dynamic removal of a behaviour at runtime                        |
| `remove_agent`                   | Reactive | Agent self-termination and AMS deregistration                   |
| `lifetime`                       | Reactive | Agents holding borrowed, lifetime-sensitive state               |
| `client_server`                  | Reactive | Two agents exchanging messages within a container               |
| `cross_platform_client_server`   | Reactive | Message exchange across platform boundaries                     |
| `esp_now_client_server`          | Reactive | ESP-NOW communication between two devices                       |
| `sensors`                        | Reactive | General demo used to present project progress                   |
| `traffic`                        | Reactive | A traffic light implemented as an FSM behaviour                 |
| `bdi_coffee`                     | BDI      | The classic "coffee maker" BDI agent, built with the Rust API    |
| `bdi_coffee_asl`                 | BDI      | The same agent written in the AgentSpeak DSL, with a sensor      |
| `bdi_rules`                      | BDI      | Derived beliefs via logic-programming rules                     |
| `bdi_rule_body_vars`             | BDI      | Rules with variables in the body                                |
| `bdi_logistics`                  | BDI      | Multi-step path planning with recursive plans                   |
| `bdi_smart_home`                 | BDI      | A context-sensitive smart-home automation agent                 |
| `bdi_send`                       | BDI      | Two BDI agents sharing beliefs over the `ember-bdil` language   |

---

## Repository Layout

```
ember/            # Façade crate: Container, MTS, feature flags, re-exports
ember-core/       # Platform-agnostic primitives (agent trait, Aid, messages, environment)
ember-reactive/   # Reactive (behaviour-based) agent architecture
ember-bdi/        # BDI agent architecture
├── bdil/         #   bit-efficient BDI content-language codec
└── macros/       #   AgentSpeak parser + proc-macro code generation
ember-fipa/       # AMS agent and FIPA agent-management ontology
ember-acc/        # Communication channel drivers (ESP-NOW, HTTP, custom)
ember-time/       # Platform time drivers
ember-util/       # Shared no_std utilities
examples/         # Runnable demonstrations
spec/             # Language & protocol specifications (e.g. ember-bdil)
docs/             # End-user documentation
```

---

## Documentation

| Document                                             | Audience     |
| ---------------------------------------------------- | ------------ |
| [`docs/`](./docs/README.md)                          | Users        |
| [`CONTRIBUTING.md`](./CONTRIBUTING.md)               | Contributors |
| [`spec/ember-bdil.md`](./spec/ember-bdil.md)         | Protocol implementers |


---

## License

To be determined. Please contact the authors before reuse or redistribution.

[`Agent`]: ./ember-core/src/agent.rs
[`Aid`]: ./ember-core/src/agent/aid.rs
[`Message`]: ./ember-core/src/message.rs
[`Content`]: ./ember-core/src/message/content.rs
