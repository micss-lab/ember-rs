# 1. Introduction

Ember is a framework for building **multi-agent systems (MAS)** that run on small, resource-constrained
embedded devices: down to microcontrollers such as the ESP32. This page explains the ideas Ember is
built on and the design choices that follow from targeting embedded hardware.

## 1.1 What is a multi-agent system?

An **agent** is an autonomous software entity that perceives its environment, holds some internal
state, and acts: including by communicating with other agents. A **multi-agent system** is a
collection of such agents that coordinate to achieve individual or shared goals.

Agents differ from ordinary tasks or threads in three ways:

- **Autonomy**: an agent decides what to do next based on its own state, not on a caller's command.
- **Communication**: agents interact primarily by exchanging messages, not by sharing memory.
- **Situatedness**: an agent runs inside an environment it can sense and affect.

These properties map naturally onto embedded and IoT scenarios: a swarm of sensor nodes, a set of
cooperating actuators, or a network of devices negotiating a shared task.

## 1.2 FIPA

Ember borrows its vocabulary and structure from the [FIPA](http://www.fipa.org/) (Foundation for
Intelligent Physical Agents) standards, the most widely adopted set of specifications for agent
platforms. The pieces Ember adopts are:

- **Agent Communication Language (ACL)**: a message format built around a *performative* (the
  communicative intent: `inform`, `request`, `query-if`, …), a receiver, an ontology, and content.
  See [Messaging](./05-messaging.md).
- **Agent Identifier (AID)**: a globally meaningful name for an agent, of the form `name@host`. In
  Ember this is the [`Aid`](../ember-core/src/agent/aid.rs) type.
- **Agent Management System (AMS)**: a privileged agent that keeps the authoritative directory of
  agents on a platform and manages their lifecycle. See [The Container](./04-container.md).
- **Message Transport Service (MTS)**: the subsystem that routes messages between agents, whether
  they live in the same container or on another device. See [Communication Channels](./09-communication-channels.md).

Ember does not aim for full FIPA compliance; it adopts the parts that pay their way on
microcontrollers and simplifies or omits the rest.

## 1.3 Agent architectures

An *agent architecture* defines how an agent turns perception and messages into action. Ember ships
two, as separate optional libraries that share the same container and messaging infrastructure:

- **Reactive (behaviour-based)** agents: a JADE-inspired model where an agent is an object that
  schedules a set of *behaviours*. Each behaviour is a small unit of logic that runs on the
  container's tick. This model is simple, predictable, and cheap. See
  [Reactive Agents](./06-reactive-agents.md).

- **BDI (Belief-Desire-Intention)** agents: a symbolic, reasoning model where an agent maintains a
  *belief base*, pursues *goals*, and executes *plans*. Ember lets you write BDI agents in an
  AgentSpeak/Jason-inspired language embedded directly in Rust. This model is more expressive and
  better suited to decision-making. See [BDI Agents](./07-bdi-agents.md).

You can mix both kinds of agent in the same container.

## 1.4 Design philosophy

Ember's design is shaped by three constraints:

1. **Embedded-first (`no_std`).** Every core crate is `no_std` and allocates through `alloc`. There
   is no operating system, no threads by default, and no assumption of a heap larger than a few tens
   of kilobytes. The `std` feature merely *adds* host conveniences; it is never required.

2. **Layered and decoupled.** Platform-agnostic primitives (the agent trait, messages, the
   environment) know nothing about containers or hardware. Agent architectures are libraries built
   on those primitives. Hardware specifics (time, communication channels) are isolated behind traits
   and feature flags. This makes it possible to port Ember to a new chip by swapping the bottom
   layer, and to add a new agent kind without touching the runtime.

3. **The execution model is decoupled from the agent implementation.** The container only knows the
   minimal [`Agent`](../ember-core/src/agent.rs) trait (`update` + `get_name`). Everything else (how
   an agent reasons, schedules work, or reacts to messages) is the agent's own business. A reactive
   agent and a BDI agent are both, to the container, just "something with an `update` method".

The result is a single-threaded, cooperatively-scheduled platform: the container repeatedly ticks
each agent, and each agent does a bounded amount of work per tick before yielding. This is a good fit
for microcontrollers, where determinism and small footprints matter more than raw parallelism.

## 1.5 Where to go next

- To get something running immediately, go to [Getting Started](./02-getting-started.md).
- To understand how the crates fit together, read [Architecture](./03-architecture.md).
