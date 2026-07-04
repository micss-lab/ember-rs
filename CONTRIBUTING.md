# Contributing to Ember

Thanks for your interest in developing Ember further. This document is the entry point for
**contributors** — people who want to change the framework itself, not just build agents with it. If
you only want to *use* Ember, the [`docs/`](./docs) directory is what you want.

Ember is research software developed as part of a master's thesis. The architecture is deliberately
modular so that new agent kinds, transports, and platform backends can be added without disturbing the
rest. This guide explains the layout, the conventions, and where to plug things in.

---

## 1. Development environment

### With Nix (recommended)

A [`flake.nix`](./flake.nix) provides a fully configured shell — the pinned Rust toolchain, the ESP
tooling (`espup`, `espflash`), `bacon`, `cargo-expand`, `cargo-flamegraph`, and more:

```sh
nix develop
```

An `.envrc` is included, so with `direnv` the shell loads automatically on `cd`.

### Without Nix

- Install [`rustup`](https://rustup.rs/). The toolchain is pinned in
  [`rust-toolchain.toml`](./rust-toolchain.toml) (`1.88.0`, with `rust-src`) and installed on first
  build.
- For ESP32 work: install the Xtensa toolchain with [`espup`](https://github.com/esp-rs/espup) and
  [`espflash`](https://github.com/esp-rs/espflash).

### Useful tools

- **`bacon`** — background checking; see [`bacon.toml`](./bacon.toml).
- **`cargo-expand`** — indispensable when working on the proc-macros (see [§6](#6-working-on-the-proc-macros)).

---

## 2. Building, checking, and testing

The default cargo target is `xtensa-esp32-none-elf`, so **always use the aliases** in
[`.cargo/config.toml`](./.cargo/config.toml). Each has a `-local` (host) and `-esp` (ESP32) variant:

| Task    | Host                 | ESP32               |
| ------- | -------------------- | ------------------- |
| build   | `cargo build-local`  | `cargo build-esp`   |
| check   | `cargo check-local`  | `cargo check-esp`   |
| clippy  | `cargo clippy-local` | `cargo clippy-esp`  |
| run     | `cargo run-local --bin <name>`  | `cargo run-esp --bin <name>` |
| test    | `cargo test-local`   | `cargo test-esp`    |

> The `-esp` aliases add `-Zbuild-std=core,alloc`. `cargo clippy` is aliased to `check-local` for a
> quick default.

**Before opening a PR**, at minimum:

```sh
cargo build-local
cargo clippy-local
cargo test-local
```

If your change touches embedded code paths, also run the `-esp` equivalents.

### Tests

- Unit and integration tests live alongside the code (`#[cfg(test)]`, `mod testing`).
- `ember-bdi/macros` uses [`trybuild`](https://docs.rs/trybuild) for compile-fail/compile-pass tests
  of the proc-macros — the right place to pin down macro diagnostics.

Currently the project is very light on tests. More tests are very much welcomed.

---

## 3. Workspace layout

Ember is a Cargo workspace ([`Cargo.toml`](./Cargo.toml)) of small, single-responsibility crates.
Dependencies point strictly downward.

| Crate            | Responsibility                                                                     | `no_std` |
| ---------------- | ---------------------------------------------------------------------------------- | -------- |
| `ember`          | Façade: `Container`, MTS wiring, AMS bootstrap, feature flags, public re-exports    | yes      |
| `ember-core`     | Primitives: `Agent` trait, `Aid`, `Environment`, FIPA ACL messages, content model  | yes      |
| `ember-reactive` | Reactive (behaviour-based) agent architecture                                      | yes      |
| `ember-bdi`      | BDI agent architecture (belief base, plans, intentions, actions, sensors)          | yes      |
| `ember-bdi/bdil` | Bit-efficient codec for the `ember-bdil` content language                          | yes      |
| `ember-bdi/macros` | Proc-macros: AgentSpeak parser + code generation, derives                        | proc-macro |
| `ember-fipa`     | AMS agent + agent-management ontology + reusable `FipaAgent` component             | yes      |
| `ember-acc`      | Communication channels (ESP-NOW, HTTP, custom) behind the `Acc` trait             | yes      |
| `ember-time`     | Platform time drivers (`std`, `esp32`)                                             | yes      |
| `ember-util`     | Shared `no_std` utilities (sync primitives, comparison helpers)                    | yes      |
| `examples`       | Runnable demonstrations for host and ESP32                                         | both     |

See [`docs/03-architecture.md`](./docs/03-architecture.md) for the user-facing architecture overview;
the rest of this section covers what contributors need to know beyond that.

### The layering rule

`ember-core` must stay free of any container, hardware, or agent-architecture concerns. It defines the
vocabulary (`Agent`, `Message`, `Environment`) that everything else builds on. If you find yourself
wanting to reference a container or a specific agent kind from `ember-core`, that is a signal the
abstraction belongs higher up.

The boundary between `ember-core` and `ember` is still being formalised — err on the side of putting
platform-agnostic types in `ember-core` and runtime/wiring in `ember`.

---

## 4. Key internal seams

Understanding these three seams makes most of the codebase navigable.

### The `Agent` trait

The container only knows [`Agent`](./ember-core/src/agent.rs): `update(&mut Environment) -> bool` and
`get_name`. Every agent kind implements it. This is the single seam that keeps reactive and BDI agents
interchangeable — respect it when adding a new kind.

### The `FipaAgent` component

Rather than inheritance, agents *embed* a [`FipaAgent`](./ember-fipa/src/agent.rs) and call its
`update` first thing each tick. It drives the `Initiated → Active` execution state and AMS
registration. A new agent kind should do the same (see how `ReactiveAgent` and `BdiAgent` use it).

### The container's tick

[`Container::poll`](./ember/src/container.rs) pumps the MTS, runs privileged agents (the AMS), then
ticks each agent once with a freshly-populated `Environment`, forwarding its outbox to the MTS. The
AMS is a **privileged agent** (`ember/src/container/privileged.rs`) that can mutate the container
directly, unlike ordinary agents.

---

## 5. How to add things

### A new reactive behaviour

Behaviours live in `ember-reactive/src/behaviour/`. Simple behaviours (`simple/`) implement an
ergonomic trait (e.g. `CyclicBehaviour`) plus an `IntoBehaviour` bridge to the internal `Behaviour`
trait. Complex behaviours (`complex/`) implement `ComplexBehaviour` and drive a `BehaviourScheduler`
over child behaviours. Follow the pattern of an existing one (`cyclic.rs` for simple, `sequential.rs`
for complex) and export it from `behaviour.rs`.

### A new communication channel

Implement [`Acc`](./ember-acc/src/lib.rs) (`send` + `receive`), gate it behind a feature, add it to
`Channels`, and expose `enable_*`/`with_*` methods on the `Container`. Serialise via the existing
representations in `ember-core::message::repr` where possible. BLE is on the roadmap and a good first
channel to attempt.

### A new content language

Add a variant to [`Content`](./ember-core/src/message/content.rs), give it a `.language()` string, and
implement encode/decode in `ember-core/src/message/repr/`. Unknown languages must continue to round-trip
as `Content::Other` rather than erroring — this is a deliberate robustness property.

### Extending `ember-bdil`

The wire format is specified in [`spec/ember-bdil.md`](./spec/ember-bdil.md) and implemented in
`ember-bdi/bdil/`. New expression/term types claim reserved code-table bytes, must be self-delimiting,
and require a **minor** version bump plus a new spec section. Keep the spec and the codec in lock-step.

### A new platform time backend

Add a feature and a driver in `ember-time/src/driver.rs` mirroring the existing `std`/`esp32` backends.

---

## 6. Working on the proc-macros

`ember-bdi/macros` is where the AgentSpeak DSL lives. It is the most intricate part of the codebase.

- **Parser.** The grammar is a [`peg`](https://docs.rs/peg) parser over a *flattened* Rust token stream
  (`token.rs`, `macros/agent.rs`). It runs on `proc_macro2` tokens, which is why programs are checked
  at compile time and errors carry spans.
- **AST → runtime.** `ast.rs` defines the parser AST and an `AstVisitor` that lowers it to token
  streams constructing the runtime types (`KnowledgeBase`, `PlanLibrary`, `Plan`, …). Built-in actions
  are parsed here (`BuiltinAction::try_from`).
- **Code generation.** `macros/agent.rs` emits the `From<Agent>`/`into_agent` impls; `macros/actions.rs`
  generates the action enum and `Execute` dispatch; `macros/derive.rs` implements `IntoLiteral`,
  `Percept`, and `FromTerm`.

Practical tips:

- Use `cargo expand` (e.g. on an example) to see generated code.
- Add a `trybuild` case for any new diagnostic or accepted/rejected syntax.
- Generated code refers to types through the `::ember::agent::bdi::…` re-export paths — keep those
  stable, or update the macros and the `ember` façade together.

Some derive/action modules are marked *"AI-generated, human verified"*; hold new code to the same bar —
verify behaviour with tests, don't just trust generation.

---

## 7. Coding conventions

- **Formatting:** `cargo fmt` (rustfmt defaults). Nix files: `nixpkgs-fmt`.
- **Linting:** keep `cargo clippy-local` clean. `clippy --fix` has been used historically; prefer
  understanding a lint before silencing it.
- **`no_std` discipline:** core crates are `#![no_std]` + `extern crate alloc`. Never reach for `std`
  outside a `#[cfg(feature = "std")]` gate. Use `alloc` collections and `core` APIs.
- **Errors, not panics:** on-device panics are fatal. Prefer returning/logging errors. A recent fix
  explicitly stopped the BDIL codec from panicking on malformed input — follow that spirit. Reserve
  panics for genuine invariant violations.
- **Logging:** use the `log` crate (`log::info!`, `debug!`, `error!`), not `println!`. Keep hot paths
  quiet.

---

## 8. Commit and PR workflow

### Commits

The history follows **Conventional Commits** with a crate scope:

```
<type>(<scope>): <summary>
```

Examples from the log:

```
feat(ember-bdi): Implement builtin action for sending messages
fix(ember-bdi/bdil): Do not panic on encoding or decoding errors
refactor(ember-core): Redo messaging to be FIPA-compatible
chore(spec): Rename ember-bdil spec file
```

Common types: `feat`, `fix`, `refactor`, `chore`, `docs`, `tests`, `examples`. Scopes are crate names
(optionally a sub-path like `ember-bdi/macros`). Keep the summary imperative and concise.

### Pull requests

- Branch off `main`; open PRs against `main`.
- Keep PRs focused; a large change is easier to review when split into `refactor` + `feat` commits.
- State what you built, why, and how you verified it. Note whether you ran the host and/or ESP checks.
- If you change the `ember-bdil` wire format, update [`spec/ember-bdil.md`](./spec/ember-bdil.md) in the
  same PR and bump the version per [§5](#extending-ember-bdil).
- If you change user-facing behaviour, update the relevant page under [`docs/`](./docs) too.

---

## 9. Where things live (quick map)

```
ember/src/container.rs              # the tick loop, MTS/AMS wiring
ember/src/container/mts.rs          # message routing (local + channels)
ember/src/container/privileged.rs   # AMS as a privileged agent
ember/src/lib.rs                    # the public re-export surface

ember-core/src/agent.rs             # the Agent trait
ember-core/src/environment.rs       # per-tick agent I/O
ember-core/src/message.rs           # Message, Content, Performative
ember-core/src/message/repr/        # bit-efficient & string wire formats

ember-reactive/src/behaviour/       # simple + complex behaviours
ember-bdi/src/                      # belief base, plans, intentions, reasoning cycle
ember-bdi/macros/src/               # the AgentSpeak DSL
ember-bdi/bdil/src/                 # the ember-bdil codec

ember-fipa/src/agent/ams.rs         # the AMS agent
ember-acc/src/                      # communication channels
spec/ember-bdil.md                  # content-language specification
docs/                               # end-user documentation
```

---

## 10. Questions

Open an issue for design discussions, bugs, or documentation gaps. Since APIs are unstable, flag
breaking changes clearly so downstream agents can adapt.

Happy hacking!
