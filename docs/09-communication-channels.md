# 9. Communication Channels

Agents on the same container talk to each other in memory, through the container's directory. To
reach an agent on **another device or container**, the MTS uses an **Agent Communication Channel
(ACC)**: a transport that serialises a message and physically delivers it. Channels live in the
`ember-acc` crate and are selected with feature flags.

## 9.1 The `Acc` trait

Every channel implements a tiny trait:

```rust
pub trait Acc {
    fn send(&mut self, aid: &Aid, message: TransportMessage) -> Result<(), ()>;
    fn receive(&mut self) -> Option<TransportMessage>;
}
```

- `send` transmits a transport message to the given remote AID.
- `receive` is polled by the MTS each `poll()`; it returns the next inbound message, if any.

Channels are aggregated in `Channels`, which the MTS owns. The container exposes builder methods to
enable each one (see [The Container §4.4](./04-container.md#44-the-message-transport-service-mts)).

## 9.2 ESP-NOW

*Feature: `acc-espnow`. Target: ESP32.*

[ESP-NOW](https://www.espressif.com/en/solutions/low-power-solutions/esp-now) is Espressif's
connectionless, low-latency peer-to-peer protocol between ESP32 devices: no Wi-Fi access point
required. It is the primary transport for on-device multi-agent deployments.

- **Addressing.** A remote agent's platform component must be a **MAC address**; the channel maps the
  AID's platform to a 6-byte MAC (`ember_acc::util::aid_to_mac`). Sending to `@local` over ESP-NOW is
  a programming error: local delivery never touches a channel.
- **Serialisation.** Messages use a custom compact encoding suited to ESP-NOW's small frame size
  (`ember-acc/src/serde/espnow.rs`), pairing naturally with the bit-efficient ACL representation.
- **Setup.** Enable it on the container with the ESP-NOW sender/receiver halves from `esp-wifi`:

  ```rust
  let container = Container::new()
      .with_espnow(Some(esp_now_sender), Some(esp_now_receiver));
  ```

See the `esp_now_client_server` and `cross_platform_client_server` examples.

## 9.3 HTTP

*Feature: `acc-http` (implies `std`). Target: hosts / Wi-Fi-connected devices.*

The HTTP channel carries messages over a Wi-Fi/IP network. It is currently `std`-only, which makes it
ideal for host-side testing and for bridging a workstation into a device network.

```rust
let mut container = Container::new();
container.enable_http(8080);          // or: Container::new().with_http(8080)
```

The channel runs a small HTTP server (built on `tiny_http`) to receive messages and uses an HTTP
client (`ureq`) to send them; multipart handling and (de)serialisation live in `ember-acc/src/http.rs`
and `ember-acc/src/serde.rs`.

## 9.4 Custom channels

*Feature: `acc-custom`.*

To add a transport Ember does not ship, such as BLE, LoRa, a serial link, or a message broker,
implement `Acc` and hand it to the container:

```rust
struct MyChannel { /* … */ }

impl ember::_crates::acc::Acc for MyChannel {
    fn send(&mut self, aid: &Aid, message: TransportMessage) -> Result<(), ()> { /* … */ }
    fn receive(&mut self) -> Option<TransportMessage> { /* … */ }
}

let container = Container::new().with_custom_acc(Box::new(MyChannel { /* … */ }));
```

Your channel is responsible for serialising the `TransportMessage` (Ember gives you the message and
its envelope stack) and for surfacing inbound messages via `receive`. Reuse the representations in
`ember-core::message::repr` (bit-efficient or string) rather than inventing a new one where possible.

## 9.5 How the MTS chooses a channel

For each receiver of an outgoing message, the MTS first tries to **resolve it locally** (including
through proxy chains). Only if the receiver is *not* a local agent does it fall through to the
channels. In the current implementation `Channels` dispatches to the single enabled channel (HTTP,
then ESP-NOW, then custom, in that order); enabling more than one transport simultaneously is not yet
fully supported.

## 9.6 A note on `std` vs `no_std`

- ESP-NOW and custom channels are `no_std`-friendly and can run on-device.
- HTTP requires `std` and therefore runs on hosts (and `std`-enabled targets), not on bare-metal
  microcontrollers.

Pick the channel that matches where your agents run; see [Embedded & ESP32](./10-embedded-esp32.md).

More channels are planned for future work. For example, LoRa and LoRa WAN.

## 9.7 Next

- [Embedded & ESP32](./10-embedded-esp32.md): running all of this on real hardware.
