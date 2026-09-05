pub(super) use esp_radio::esp_now::{EspNowReceiver, EspNowSender};

use alloc::boxed::Box;
use alloc::collections::VecDeque;
use alloc::format;
use alloc::vec::Vec;
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll, Waker};

use esp_radio::esp_now::{EspNowError, EspNowReceiver as Receiver, EspNowSender as Sender};

use ember_core::agent::aid::Aid;
use ember_core::environment::Environment;
use ember_core::message::repr::payload::bit_efficient;
use ember_core::message::{
    AclRepresentation, MessageEnvelope, MessageEnvelopes, Payload, TransportMessage,
};

use crate::util::aid_to_mac;
use crate::{Acc, SendCallbacks, SendError};

use self::reliable::{
    PendingSend, RELIABLE_FRAME_OVERHEAD, ReliableFrame, ReliableFrameOut, ReliableState,
};

mod reliable;

type InFlight<'c> = Pin<Box<dyn Future<Output = (Sender<'c>, Result<(), EspNowError>)> + 'c>>;

const MAX_QUEUE_LEN: usize = 32;
const MAX_PRIORITY_QUEUE_LEN: usize = 32;
const MAX_CONSECUTIVE_PRIORITY_SENDS: u32 = 3;

enum QueueEntry {
    PlainData {
        // Aid is only stored for logging purposes.
        aid: Option<Aid>,
        mac: [u8; 6],
        frame: Box<[u8]>,
        callbacks: SendCallbacks,
    },
    ReliableData {
        seq: u32,
    },
    Ack {
        mac: [u8; 6],
        seq: u32,
    },
}

enum InFlightMeta {
    Plain {
        aid: Option<Aid>,
        callbacks: SendCallbacks,
    },
    ReliableData {
        seq: u32,
    },
    Ack,
}

/// An espnow channel.
///
/// # Reliable
///
/// With reliable disabled, sends are fire and forget: no acks are
/// sent or expected. `on_success`/`on_failure` only reflect the espnow driver
/// accepting or rejecting the send, not whether the peer received the frame.
/// With reliable enabled, sends are tracked and retried, and on_success only
/// fires once the peer's ack comes back.
pub struct EspNowChannel<'c> {
    sender: Option<Sender<'c>>,
    receiver: Option<Receiver<'c>>,

    queue: VecDeque<QueueEntry>,
    priority_queue: VecDeque<QueueEntry>,
    consecutive_priority_sends: u32,

    in_flight: Option<InFlight<'c>>,
    in_flight_meta: Option<InFlightMeta>,

    reliable_state: Option<ReliableState>,
}

impl<'c> EspNowChannel<'c> {
    pub fn new(sender: Option<Sender<'c>>, receiver: Option<Receiver<'c>>, reliable: bool) -> Self {
        Self {
            sender,
            receiver,
            queue: VecDeque::new(),
            priority_queue: VecDeque::new(),
            consecutive_priority_sends: 0,
            in_flight: None,
            in_flight_meta: None,
            reliable_state: reliable.then(ReliableState::new),
        }
    }

    /// Enables or disables the reliability layer with handling of still queued
    /// or in-flight messges. Disabling fires `on_failure` and `on_complete`
    /// for any send still waiting on an ack, instead of abandoning it
    /// silently.
    pub fn set_reliable(&mut self, reliable: bool, environment: &mut Environment) {
        match (reliable, self.reliable_state.take()) {
            (true, state) => self.reliable_state = Some(state.unwrap_or_else(ReliableState::new)),
            (false, Some(state)) => state.drain(environment),
            (false, None) => {}
        }
    }

    // TODO: Remove priority dirty fix.
    fn enqueue(&mut self, entry: QueueEntry, priority: bool) {
        if self.sender.is_none() && self.in_flight.is_none() {
            return;
        }
        self.make_room(priority);
        if priority {
            self.priority_queue.push_back(entry);
        } else {
            self.queue.push_back(entry);
        }
    }

    fn make_room(&mut self, priority: bool) {
        let (queue, max) = if priority {
            (&mut self.priority_queue, MAX_PRIORITY_QUEUE_LEN)
        } else {
            (&mut self.queue, MAX_QUEUE_LEN)
        };
        if queue.len() < max {
            return;
        }
        let dropped = queue.pop_front();
        log::warn!("EspNow send queue full ({max}), dropping oldest queued send");
        if let Some(QueueEntry::ReliableData { seq }) = dropped
            && let Some(state) = self.reliable_state.as_mut()
        {
            state.pending.remove(&seq);
        }
    }

    fn next_entry(&mut self) -> Option<QueueEntry> {
        let take_priority = !self.priority_queue.is_empty()
            && (self.queue.is_empty()
                || self.consecutive_priority_sends < MAX_CONSECUTIVE_PRIORITY_SENDS);

        if take_priority {
            self.consecutive_priority_sends += 1;
            return self.priority_queue.pop_front();
        }

        self.consecutive_priority_sends = 0;
        self.queue
            .pop_front()
            .or_else(|| self.priority_queue.pop_front())
    }

    fn poll_send(&mut self, environment: &mut Environment) {
        if self.in_flight.is_none() {
            let Some(sender) = self.sender.take() else {
                return;
            };

            let (mac, frame, meta) = loop {
                let Some(entry) = self.next_entry() else {
                    self.sender = Some(sender);
                    return;
                };
                match entry {
                    QueueEntry::PlainData {
                        aid,
                        mac,
                        frame,
                        callbacks,
                    } => {
                        break (mac, frame, InFlightMeta::Plain { aid, callbacks });
                    }
                    QueueEntry::Ack { mac, seq } => {
                        let frame = postcard::to_allocvec(&ReliableFrameOut::Ack { seq })
                            .expect("failed to serialize reliable frame");
                        break (mac, frame.into_boxed_slice(), InFlightMeta::Ack);
                    }
                    QueueEntry::ReliableData { seq } => {
                        let Some(pending) = self
                            .reliable_state
                            .as_ref()
                            .and_then(|state| state.pending.get(&seq))
                        else {
                            continue;
                        };
                        let frame = postcard::to_allocvec(&ReliableFrameOut::Data {
                            seq,
                            inner: &pending.inner,
                        })
                        .expect("failed to serialize reliable frame");
                        break (
                            pending.mac,
                            frame.into_boxed_slice(),
                            InFlightMeta::ReliableData { seq },
                        );
                    }
                }
            };

            self.in_flight = Some(Box::pin(send_frame(sender, mac, frame)));
            self.in_flight_meta = Some(meta);
        }

        let waker = Waker::noop();
        let mut cx = Context::from_waker(waker);
        let Poll::Ready((sender, result)) = self.in_flight.as_mut().unwrap().as_mut().poll(&mut cx)
        else {
            return;
        };

        self.sender = Some(sender);
        self.in_flight = None;

        match self.in_flight_meta.take() {
            Some(InFlightMeta::Plain { aid, mut callbacks }) => {
                // Send and forget: on_success/on_failure just reflect whether the espnow
                // driver accepted the send, not whether the peer received it.
                match result {
                    Ok(()) => {
                        if let Some(on_success) = callbacks.on_success.take() {
                            on_success(environment);
                        }
                    }
                    Err(err) => {
                        log::error!("EspNow send error to {aid:?}: {err:?}");
                        if let Some(on_failure) = callbacks.on_failure.take() {
                            on_failure(environment);
                        }
                    }
                }
                if let Some(on_complete) = callbacks.on_complete.take() {
                    on_complete(environment);
                }
            }
            Some(InFlightMeta::ReliableData { seq }) => {
                if let Err(err) = result {
                    log::error!("EspNow send error (seq {seq}): {err:?}");
                }
                if let Some(state) = self.reliable_state.as_mut()
                    && let Some(pending) = state.pending.get_mut(&seq)
                {
                    pending.sent_at = Some(ember_time::now());
                }
            }
            Some(InFlightMeta::Ack) => {
                if let Err(err) = result {
                    log::error!("EspNow send error (ack): {err:?}");
                }
            }
            None => {}
        }
    }
}

async fn send_frame<'c>(
    mut sender: Sender<'c>,
    mac: [u8; 6],
    frame: Box<[u8]>,
) -> (Sender<'c>, Result<(), EspNowError>) {
    let result = sender.send_async(&mac, &frame).await;
    (sender, result)
}

#[derive(serde::Serialize)]
struct EspNowMessageOut<'a> {
    to: &'a [Aid],
    from: &'a Aid,
    content: &'a [u8],
}

#[derive(serde::Deserialize)]
struct EspNowMessageIn<'a> {
    to: Vec<Aid>,
    from: Aid,
    content: &'a [u8],
}

fn encode_transport(message: &TransportMessage) -> Vec<u8> {
    let envelope = &message.envelopes.base;
    let content = match &message.payload {
        Payload::AclMessage(m) => bit_efficient::encode(m),
        // TODO: implement this.
        Payload::Bytes(_) => unimplemented!(),
    };
    postcard::to_allocvec(&EspNowMessageOut {
        to: &envelope.to,
        from: &envelope.from,
        content: &content,
    })
    .expect("failed to serialize message into postcard data format")
}

fn decode_message(bytes: &[u8]) -> Option<TransportMessage> {
    let message = postcard::from_bytes::<EspNowMessageIn>(bytes)
        .inspect_err(|_| log::trace!("skipping unparsable message"))
        .ok()?;
    let payload = bit_efficient::decode(message.content).ok()?;
    Some(TransportMessage {
        envelopes: MessageEnvelopes {
            base: MessageEnvelope {
                to: message.to,
                from: message.from,
                date: chrono::DateTime::<chrono::Utc>::MIN_UTC.into(),
                acl_representation: AclRepresentation::BitEfficient,
                other: None,
            },
            others: Vec::with_capacity(0),
        },
        payload: Payload::AclMessage(payload),
    })
}

impl<'c> Acc for EspNowChannel<'c> {
    fn send(
        &mut self,
        address: &Aid,
        message: TransportMessage,
        callbacks: SendCallbacks,
        _environment: &mut Environment,
    ) -> Result<(), SendError> {
        if self.sender.is_none() && self.in_flight.is_none() {
            return Err(SendError::Generic(
                "espnow channel is not configured for sending messages".into(),
            ));
        }

        let mac = aid_to_mac(address);
        let inner = encode_transport(&message);

        if inner.len()
            + if self.reliable_state.is_some() {
                RELIABLE_FRAME_OVERHEAD
            } else {
                0
            }
            > esp_radio::esp_now::ESP_NOW_MAX_DATA_LEN
        {
            return Err(SendError::Generic(
                format!(
                    "message to {mac:02x?} is {} bytes, over ESP-NOW's {}-byte limit, dropping",
                    inner.len(),
                    esp_radio::esp_now::ESP_NOW_MAX_DATA_LEN
                )
                .into(),
            ));
        }

        match self.reliable_state.as_mut() {
            Some(state) => {
                let seq = state.next_seq;
                state.next_seq = state.next_seq.wrapping_add(1);
                state.pending.insert(
                    seq,
                    PendingSend {
                        mac,
                        inner: inner.into_boxed_slice(),
                        attempts: 1,
                        sent_at: None,
                        callbacks,
                    },
                );
                self.enqueue(QueueEntry::ReliableData { seq }, false);
            }
            None => {
                self.enqueue(
                    QueueEntry::PlainData {
                        aid: Some(address.clone()),
                        mac,
                        frame: inner.into_boxed_slice(),
                        callbacks,
                    },
                    false,
                );
            }
        }

        Ok(())
    }

    fn receive(&mut self, environment: &mut Environment) -> Option<TransportMessage> {
        if let Some(state) = self.reliable_state.as_mut() {
            let due = state.retry_due_at(ember_time::now(), environment);
            for seq in due {
                self.enqueue(QueueEntry::ReliableData { seq }, false);
            }
        }

        loop {
            self.poll_send(environment);
            let message = self.receiver.as_mut()?.receive()?;
            let bytes = message.data();
            let src_mac = message.info.src_address;

            if self.reliable_state.is_none() {
                return decode_message(bytes);
            }

            let frame = match postcard::from_bytes::<ReliableFrame>(bytes) {
                Ok(frame) => frame,
                Err(_) => {
                    log::trace!("reliable espnow: skipping unparsable frame");
                    continue;
                }
            };

            match frame {
                ReliableFrame::Ack { seq } => {
                    if let Some(state) = self.reliable_state.as_mut() {
                        state.handle_ack(seq, environment);
                    }
                    continue;
                }
                ReliableFrame::Data { seq, inner } => {
                    self.enqueue(QueueEntry::Ack { mac: src_mac, seq }, true);
                    let accepted = self
                        .reliable_state
                        .as_mut()
                        .is_some_and(|state| state.accept(src_mac, seq));
                    if !accepted {
                        continue;
                    }
                    return decode_message(&inner);
                }
            }
        }
    }
}
