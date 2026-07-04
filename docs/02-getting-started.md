# 2. Getting Started

This page takes you from a clean checkout to a running agent: first on your host machine, then on an
ESP32.

## 2.1 Prerequisites

- **Rust `1.88.0`.** The exact toolchain is pinned in
  [`rust-toolchain.toml`](../rust-toolchain.toml); `rustup` will install it automatically the first
  time you build. The `rust-src` component is required (it is listed in the toolchain file).
- **For ESP32 targets:** the Xtensa Rust toolchain, installed with
  [`espup`](https://github.com/esp-rs/espup), plus [`espflash`](https://github.com/esp-rs/espflash)
  for flashing and monitoring.
- **Optional but recommended:** [Nix](https://nixos.org/). The repository ships a `flake.nix` that
  provides a complete development shell (Rust, `espup`, `espflash`, `bacon`, and more):

  ```sh
  nix develop
  ```

## 2.2 The two targets and the `-local` aliases

The default compile target for this workspace is **`xtensa-esp32-none-elf`**: it is set in
[`.cargo/config.toml`](../.cargo/config.toml). That means a bare `cargo build` tries to build for the
ESP32.

To build and run on your **host** machine instead, use the cargo aliases defined in the same file.
They target `x86_64-unknown-linux-gnu`:

| Host alias          | ESP32 alias       | Plain equivalent                              |
| ------------------- | ----------------- | --------------------------------------------- |
| `cargo build-local` | `cargo build-esp` | `cargo build --target …`                      |
| `cargo run-local`   | `cargo run-esp`   | `cargo run --target …`                        |
| `cargo check-local` | `cargo check-esp` | `cargo check --target …`                      |
| `cargo clippy-local`| `cargo clippy-esp`| `cargo clippy --target …`                     |
| `cargo test-local`  | `cargo test-esp`  | `cargo test --target …`                       |

The `-esp` aliases additionally pass `-Zbuild-std=core,alloc`, because there is no pre-built `core`
for the Xtensa bare-metal target.

## 2.3 Building

```sh
# Everything, on the host
cargo build-local

# Everything, for the ESP32 (needs the xtensa toolchain)
cargo build-esp
```

## 2.4 Running your first example

Examples are ordinary binaries under [`examples/src/bin/`](../examples/src/bin). Run one on the host
with `run-local --bin <name>`:

```sh
cargo run-local --bin behaviour_cyclic
```

You should see log output from an agent ticking a cyclic behaviour. Try a BDI example next:

```sh
cargo run-local --bin bdi_coffee_asl
```

Logging is initialised for you by the `setup_example!()` macro; the default level is `TRACE`. See
[Examples](./11-examples.md) for a description of each one.

## 2.5 Your first agent (host)

A minimal reactive agent is a struct implementing a behaviour trait, added to a `Container`:

```rust
use ember::Container;
use ember::agent::reactive::ReactiveAgent;
use ember::agent::reactive::behaviour::{Context, CyclicBehaviour};

struct Hello;

impl CyclicBehaviour for Hello {
    type AgentState = ();
    type Event = ();

    fn action(&mut self, _ctx: &mut Context<Self::Event>, _state: &mut Self::AgentState) {
        log::info!("Hello from an agent!");
    }

    fn is_finished(&self) -> bool {
        true // run once, then finish
    }
}

fn main() {
    Container::new()
        .with_agent(ReactiveAgent::new("greeter", ()).with_behaviour(Hello))
        .start()
        .expect("container error");
}
```

`Container::start()` runs the cooperative scheduler until the platform is stopped or every agent has
finished. See [The Container](./04-container.md) for the details of that loop.

## 2.6 Flashing to an ESP32

With a board connected over USB, the `run-esp` alias builds for Xtensa and (via the `runner`
configured in `.cargo/config.toml`) flashes and opens a serial monitor with `espflash`:

```sh
cargo run-esp --bin sensors
```

For running under the [Wokwi](https://wokwi.com/) simulator instead of physical hardware, see
[Embedded & ESP32](./10-embedded-esp32.md).

## 2.7 Handy tooling

- **`bacon`**: a background code checker; a [`bacon.toml`](../bacon.toml) is provided.

## 2.8 Next steps

- [Architecture](./03-architecture.md): how the crates fit together.
- [Reactive Agents](./06-reactive-agents.md) / [BDI Agents](./07-bdi-agents.md): build something real.
