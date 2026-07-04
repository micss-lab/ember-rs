# Ember Documentation

Welcome to the documentation for **Ember**, the embedded-first multi-agent platform. These pages
describe everything an end user needs to build, run, and deploy multi-agent systems with Ember: from
first principles to the full BDI language reference.

> Ember is research software. APIs are unstable and may change between versions. Where this
> documentation and the source disagree, the source is authoritative; please
> [open an issue](../CONTRIBUTING.md) so we can fix the docs.

## Reading order

If you are new, read the guides in order. If you already know what you are looking for, jump straight
to the relevant reference.

| #  | Document                                                     | What it covers                                                        |
| -- | ------------------------------------------------------------ | --------------------------------------------------------------------- |
| 01 | [Introduction](./01-introduction.md)                         | What a multi-agent system is, FIPA, and Ember's design philosophy     |
| 02 | [Getting Started](./02-getting-started.md)                   | Installing the toolchain, building, and running your first agent      |
| 03 | [Architecture](./03-architecture.md)                         | The crate stack and how the layers fit together                       |
| 04 | [The Container](./04-container.md)                           | The runtime: scheduling, the tick loop, the AMS and MTS               |
| 05 | [Messaging](./05-messaging.md)                               | FIPA ACL messages, content languages, and wire representations        |
| 06 | [Reactive Agents](./06-reactive-agents.md)                   | Behaviour-based agents: simple and complex behaviours, the context    |
| 07 | [BDI Agents](./07-bdi-agents.md)                             | Belief-Desire-Intention agents and the AgentSpeak-like language       |
| 08 | [The `ember-bdil` Content Language](./08-bdil.md)            | Sharing belief state between agents over the wire                     |
| 09 | [Communication Channels](./09-communication-channels.md)     | ESP-NOW, HTTP, and writing a custom channel                           |
| 10 | [Embedded & ESP32](./10-embedded-esp32.md)                   | `no_std`, targets, flashing, and simulation                           |
| 11 | [Examples](./11-examples.md)                                 | A guided tour of every example in the repository                      |
|    | [Glossary](./glossary.md)                                    | Definitions of the terms used throughout                              |

## Quick links

- **Just want to run something?** → [Getting Started](./02-getting-started.md)
- **Building reactive agents?** → [Reactive Agents](./06-reactive-agents.md)
- **Building BDI agents?** → [BDI Agents](./07-bdi-agents.md)
- **Contributing to Ember itself?** → [`CONTRIBUTING.md`](../CONTRIBUTING.md)
