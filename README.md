# `ember` - The embedded-first multi-agent platform

**Ember** is a multi-agent framework for resource-constrained embedded devices, written in Rust.
It provides the infrastructure for deploying communicating agents on microcontrollers, with the
ESP32 as the primary target. The design follows a layered architecture, separating
platform-agnostic abstractions from device-specific implementations, and keeping the agent
execution model decoupled from the agent implementation.

The framework draws on concepts from the [FIPA](http://www.fipa.org/) agent standards, including
the Agent Management System (AMS) and Message Transport Service (MTS), adapted for the constraints
of embedded environments.

> This project is research software. APIs are unstable and subject to change.

---

## Architecture

Currently the separation between `ember-core` and `ember` is an arbitrary division. We plan to
improve this separation with the introduction of new agent types (e.g. bdi-agents).

```
┌───────────────────────────────────────────────────────────────────────────────────────┐
│                                         ember                                         │
│                         Container · reactive agents · AMS/MTS                         │
├───────────────────────────────────────────────────────────────────────────────────────┤
│                                 agent kind libraries                                  │
│                <currently in core> ember-reactive (planned: ember-bdi)                │
├─────────────────────────────────────┬────────────────┬────────────────────────────────┤
│             ember-core              │    ember-acc   │           ember-time           │
│ agent-trait · behaviours · messages │ esp-now · http │ platform-specific time drivers │
└─────────────────────────────────────┴────────────────┴────────────────────────────────┘
```

The codebase is split into several crates:

### `ember-core`

The platform-agnostic foundation. Defines:

- **Agent Entity** - (`AgentID`) and the agent trait.
- **Message structure** - envelope, content, language (SL), ontologies, and serialisation/deserialisation.
- **Behaviours** - The execution primitives attached to reactive agents:
  - *Simple*: `Cyclic`, `OneShot`, `Ticker`
  - *Complex*: `FSM`, `Parallel`, `Sequential`, `Blocked`
- **Agent context** - The agent's window to interact with the outside world and other agents.

### `ember`

The layer that assembles `ember-core` into a runnable framework. Provides:

- **Container** - The runtime that hosts agents and dispatches messages, including an MTS.
  implementation for intra-container delivery
- **Agent Management System (AMS)** - registers, lifecycle-manages, and terminates agents
- **Reacive Agents** - Implementation of the `Agent` trait for reactive (behaviour-based) agents.

The boundary between `ember-core` and `ember` is still being formalised; the intent is that
`ember-core` remains free of any container or platform concerns.

### `ember-acc`

Agent Communication Channel implementations. Currently:

- **ESP-NOW** - low-latency peer-to-peer communication between ESP32 devices, with custom
  serialisation
- **HTTP** - for communication over Wi-Fi networks (currently only supported on std-enabled targets).

### `ember-time`

Platform-specific time driver implementations. Provides the tick sources and delay abstractions
used by `Ticker` behaviours and other time-sensitive components.

---

## Agent Kinds

ember is designed to support multiple agent architectures as separate, optional libraries. Each
kind can be composed with the same underlying container and communication infrastructure.

| Crate | Kind |
|---|---|
| `ember` | Reactive (behaviour-based) |

### Reactive (behaviour-based) Agents

The behaviour-based agents in `ember` follow a JADE-inspired model: agents are objects that
execute a set of scheduled behaviours, with message passing as the primary coordination mechanism.

---

## Getting Started

### Prerequisites

- Rust (see `rust-toolchain.toml` for the required toolchain version)
- For ESP32 targets: the `xtensa` or `riscv` toolchain as appropriate for your chip variant

### Building

```sh
cargo build
```

To build and run the examples on a local target:

```sh
cargo run --example behaviour_cyclic
cargo run --example client_server
cargo run --example bdi_demo
```

For ESP32 targets, see the `wokwi.toml` and `examples/build.rs` for the build configuration used
in the examples.

### Examples

The `examples/` directory contains a set of programs that demonstrate individual features:

| Example | Description |
|---|---|
| `behaviour_cyclic` | A simple agent with a repeating behaviour |
| `behaviour_oneshot` | A behaviour that executes once and terminates |
| `behaviour_fsm` | Finite state machine behaviour |
| `client_server` | Two agents exchanging messages within a container |
| `cross_platform_client_server` | Message exchange across platform boundaries |
| `esp_now_client_server` | ESP-NOW-based communication between two devices |
| `block_behaviour` | Blocking a behaviour waiting for a message |
| `remove_behaviour` | Dynamic removal of a behaviour at runtime |
| `remove_agent` | Agent self-termination and AMS deregistration (no guaranteed support for the feature) |
| `lifetime` | Agents with lifetime-sensitive state |
| `sensors` | General demo used to present project progress |
| `traffic` | Traffic light implemented using an fsm behaviour |

---

## Repository Layout

```
ember/           # Main framework (container, AMS, MTS, FIPA types)
ember-core/      # Platform-agnostic primitives (behaviours, messages, agent trait)
ember-acc/       # Communication channel drivers
ember-time/      # Platform time drivers
examples/        # Runnable demonstrations
```

---

## Status and Roadmap

ember is research software developed as part of an investigation into multi-agent systems for
embedded and IoT environments. Current priorities:

- Stabilise the `ember` / `ember-core` interface.
- Complete the `ember-bdi` agent implementation.
- Add additional communication channels (BLE).

---

## License

To be determined. Please contact the authors before reuse or redistribution.
