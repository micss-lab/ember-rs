# 10. Embedded & ESP32

Ember is embedded-first: every core crate is `no_std` and the default build target is the ESP32. This
page covers building for hardware, flashing, and simulating.

## 10.1 `no_std` and `alloc`

All of Ember's core crates are `#![no_std]` and allocate through `alloc`. There is no assumption of an
operating system, threads, or a large heap. The `std` feature only *adds* host conveniences (the HTTP
channel, host time driver, richer logging): it is never required on-device.

On the ESP32 you must provide a global allocator. The examples do this with `esp-alloc`, reserving a
modest heap:

```rust
// examples/src/esp.rs
const HEAP_SIZE: usize = 72 * 1024;

pub fn init_heap() {
    esp_alloc::heap_allocator!(HEAP_SIZE);
}
```

An Ember application's `main` on the ESP32 is a bare-metal entry point: it initialises logging and the
heap, then constructs and starts a `Container`. The examples' `setup_example!()` macro generates the
right `main` for both host and device automatically (see `examples/src/lib.rs`).

## 10.2 Targets and toolchains

| Target                     | Use              | Toolchain                                             |
| -------------------------- | ---------------- | ---------------------------------------------------- |
| `x86_64-unknown-linux-gnu` | host development | the pinned stable toolchain                          |
| `xtensa-esp32-none-elf`    | ESP32 hardware   | the Xtensa toolchain from [`espup`](https://github.com/esp-rs/espup) |

The default target is `xtensa-esp32-none-elf` (`.cargo/config.toml`). Use the `-local` / `-esp` cargo
aliases to pick explicitly: see [Getting Started §2.2](./02-getting-started.md#22-the-two-targets-and-the-local-aliases).
The `-esp` aliases pass `-Zbuild-std=core,alloc` because the bare-metal target has no pre-built `core`.

Relevant features when building for the ESP32:

| Feature      | Purpose                                              |
| ------------ | --------------------------------------------------- |
| `esp32`      | Selects the ESP32 time driver (`ember-time/esp32`). |
| `acc-espnow` | Enables ESP-NOW communication.                      |

The example crate wires these up per-target in its `Cargo.toml` (host builds pull `acc-http`; `none`
builds pull `acc-espnow`, `esp-hal`, `esp-wifi`, and the ESP runtime crates).

## 10.3 Building and flashing

Build for the ESP32:

```sh
cargo build-esp --bin sensors
```

Flash and monitor a connected board: the `runner` in `.cargo/config.toml` is
`espflash flash --monitor`, so the `run-esp` alias does it in one step:

```sh
cargo run-esp --bin sensors
```

You need [`espflash`](https://github.com/esp-rs/espflash) installed and the board on a serial port.
The `flake.nix` dev shell provides `espflash`, `espup`, and everything else.

## 10.4 Simulation with Wokwi

You can run firmware in the [Wokwi](https://wokwi.com/) ESP32 simulator without hardware. The repo
includes:

- [`examples/wokwi.toml`](../examples/wokwi.toml): points Wokwi at a built ELF, e.g.
  `../target/xtensa-esp32-none-elf/release/esp_now_client_server`.
- [`examples/diagram.json`](../examples/diagram.json): a minimal ESP32 dev-kit board wired to the
  serial monitor.

Build the target firmware first (a release build, to match the path in `wokwi.toml`), then start the
simulation with the Wokwi tooling / VS Code extension pointed at `examples/wokwi.toml`.

## 10.5 Memory and performance notes

- **Bounded work per tick.** The cooperative scheduler ticks each agent once per round; keep each
  behaviour/reasoning step short so the platform stays responsive.
- **Prefer bit-efficient messages.** On constrained links use the bit-efficient ACL representation
  (the default) and the `ember-bdil` content language for belief sharing: both are designed for
  minimal footprint.
- **Watch allocations.** With a ~72 KB heap, avoid unbounded growth in belief bases, plan libraries,
  and inboxes.
- **Build profiles.** The workspace defines a `bindings` profile (`opt-level = "z"`, LTO, stripped)
  for size-optimised artefacts; see the root `Cargo.toml`.

## 10.6 Next

- [Examples](./11-examples.md): including the ESP-NOW and cross-platform demos.
