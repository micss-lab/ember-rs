use alloc::boxed::Box;
use alloc::collections::VecDeque;
use alloc::vec::Vec;
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll, Waker};

use ember_core::agent::aid::Aid;
use ember_core::environment::Environment;
use ember_core::message::repr::payload::bit_efficient;
use ember_core::message::{Payload, TransportMessage};
use ember_time::{Duration, Instant};

use ember_collections::SmallMap;

use esp_radio::esp_now::{EspNowError, EspNowReceiver as Receiver, EspNowSender as Sender};

use crate::serde::espnow::de::EspNowMessageDe;
use crate::serde::espnow::ser::EspNowMessageSer;
use crate::{Acc, SendCallbacks};

use super::{EspNowReceiver, EspNowSender};

const RETRY_BASE_INTERVAL: Duration = Duration::millis(700);
const MAX_ATTEMPTS: u32 = 5;
const MAX_QUEUE_LEN: usize = 32;
const MAX_PRIORITY_QUEUE_LEN: usize = 32;
const MAX_CONSECUTIVE_PRIORITY_SENDS: u32 = 3;
// Messages are stored as rust objects, requiring some math to check payload size is under the max.
const RELIABLE_FRAME_OVERHEAD: usize = 8;

type InFlight<'c> = Pin<Box<dyn Future<Output = (Sender<'c>, Result<(), EspNowError>)> + 'c>>;

#[derive(serde::Serialize, serde::Deserialize)]
enum ReliableFrame {
    Data { seq: u32, inner: Vec<u8> },
    Ack { seq: u32 },
}

#[derive(serde::Serialize)]
enum ReliableFrameOut<'a> {
    Data { seq: u32, inner: &'a [u8] },
    Ack { seq: u32 },
}

enum QueueEntry {
    Data { seq: u32 },
    Ack { mac: [u8; 6], seq: u32 },
}

struct PendingSend {
    mac: [u8; 6],
    inner: Box<[u8]>,
    attempts: u32,
    sent_at: Option<Instant>,
    callbacks: SendCallbacks,
}

impl PendingSend {
    /// Exponential backoff duration to wait for before retrying.
    fn retry_after(&self) -> Duration {
        let shift = (self.attempts - 1).min(MAX_ATTEMPTS - 1);
        RETRY_BASE_INTERVAL * (1u32 << shift)
    }
}

/// Seq tracker per mac that has ever sent to this channel.
struct SeenSender {
    mac: [u8; 6],
    highest_seq: u32,
    /// Bitmask of recently accepted sequence numbers older than `highest_seq`, to tolerate reordering from requeued retries.
    window: u32,
}

pub struct ReliableEspNowChannel<'c> {
    sender: Option<Sender<'c>>,
    receiver: Option<Receiver<'c>>,
    queue: VecDeque<QueueEntry>,
    priority_queue: VecDeque<QueueEntry>,
    in_flight: Option<InFlight<'c>>,
    in_flight_seq: Option<u32>,
    consecutive_priority_sends: u32,
    reliable_peers: Box<[[u8; 6]]>,
    next_seq: u32,
    pending: SmallMap<u32, PendingSend>,
    seen: Vec<SeenSender>,
}

impl<'c> ReliableEspNowChannel<'c> {
    pub fn new(
        sender: Option<EspNowSender<'c>>,
        receiver: Option<EspNowReceiver<'c>>,
        reliable_peers: impl IntoIterator<Item = [u8; 6]>,
    ) -> Self {
        Self {
            sender,
            receiver,
            queue: VecDeque::new(),
            priority_queue: VecDeque::new(),
            in_flight: None,
            in_flight_seq: None,
            consecutive_priority_sends: 0,
            reliable_peers: reliable_peers.into_iter().collect(),
            next_seq: 0,
            pending: SmallMap::new(),
            seen: Vec::new(),
        }
    }

    fn retry_due(&mut self, environment: &mut Environment) {
        self.retry_due_at(ember_time::now(), environment);
    }

    // Split out so tests can drive it without waiting on real wall-clock time.
    fn retry_due_at(&mut self, now: Instant, environment: &mut Environment) {
        let due: Vec<u32> = self
            .pending
            .iter()
            .filter(|(_, entry)| entry.attempts < MAX_ATTEMPTS)
            .filter_map(|(seq, entry)| {
                let sent_at = entry.sent_at?;
                (now - sent_at >= entry.retry_after()).then_some(*seq)
            })
            .collect();

        for seq in due {
            let Some(entry) = self.pending.get_mut(&seq) else {
                continue;
            };
            entry.attempts += 1;
            entry.sent_at = None;
            log::debug!(
                "reliable espnow: retrying seq {seq} (attempt {})",
                entry.attempts
            );
            if let Some(on_retry) = entry.callbacks.on_retry.as_mut() {
                on_retry(entry.attempts, environment);
            }
            self.enqueue_data(seq);
        }

        let mut gave_up = 0usize;
        let mut failures = Vec::new();
        self.pending.retain(|seq, entry| {
            let exhausted = entry.attempts >= MAX_ATTEMPTS;
            if exhausted {
                log::warn!(
                    "reliable espnow: giving up on seq {seq} after {MAX_ATTEMPTS} attempts, no ack received"
                );
                gave_up += 1;
                failures.push((entry.callbacks.on_failure.take(), entry.callbacks.on_complete.take()));
            }
            !exhausted
        });
        for (on_failure, on_complete) in failures {
            if let Some(on_failure) = on_failure {
                on_failure(environment);
            }
            if let Some(on_complete) = on_complete {
                on_complete(environment);
            }
        }
    }

    // Split out so tests can drive it without a real ack frame arriving over
    // the radio.
    fn handle_ack(&mut self, seq: u32, environment: &mut Environment) {
        let Some(mut entry) = self.pending.remove(&seq) else {
            return;
        };
        if let Some(on_success) = entry.callbacks.on_success.take() {
            on_success(environment);
        }
        if let Some(on_complete) = entry.callbacks.on_complete.take() {
            on_complete(environment);
        }
    }

    fn send_ack(&mut self, mac: [u8; 6], seq: u32) {
        if self.sender.is_none() && self.in_flight.is_none() {
            return;
        }
        self.make_room(true);
        self.priority_queue.push_back(QueueEntry::Ack { mac, seq });
    }

    /// Message deduplication ensuring a message arrives at most a single time.
    fn accept(&mut self, mac: [u8; 6], seq: u32) -> bool {
        let Some(entry) = self.seen.iter_mut().find(|s| s.mac == mac) else {
            self.seen.push(SeenSender {
                mac,
                highest_seq: seq,
                window: 0,
            });
            return true;
        };

        if (seq.wrapping_sub(entry.highest_seq) as i32) > 0 {
            let shift = seq.wrapping_sub(entry.highest_seq);
            entry.window = if shift >= 32 {
                0
            } else {
                (entry.window << shift) | (1 << (shift - 1))
            };
            entry.highest_seq = seq;
            return true;
        }

        let diff = entry.highest_seq.wrapping_sub(seq);
        if diff == 0 || diff > 32 {
            return false;
        }
        let bit = 1u32 << (diff - 1);
        if entry.window & bit != 0 {
            false
        } else {
            entry.window |= bit;
            true
        }
    }

    fn enqueue_data(&mut self, seq: u32) {
        self.make_room(false);
        self.queue.push_back(QueueEntry::Data { seq });
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
        if let Some(QueueEntry::Data { seq }) = dropped {
            self.pending.remove(&seq);
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

    fn poll_send(&mut self) {
        if self.in_flight.is_none() {
            let Some(sender) = self.sender.take() else {
                return;
            };

            let (mac, frame, seq) = loop {
                let Some(entry) = self.next_entry() else {
                    self.sender = Some(sender);
                    return;
                };
                match entry {
                    QueueEntry::Ack { mac, seq } => {
                        let frame = postcard::to_allocvec(&ReliableFrameOut::Ack { seq })
                            .expect("failed to serialize reliable frame");
                        break (mac, frame.into_boxed_slice(), None);
                    }
                    QueueEntry::Data { seq } => {
                        let Some(pending) = self.pending.get(&seq) else {
                            continue;
                        };
                        let frame = postcard::to_allocvec(&ReliableFrameOut::Data {
                            seq,
                            inner: &pending.inner,
                        })
                        .expect("failed to serialize reliable frame");
                        break (pending.mac, frame.into_boxed_slice(), Some(seq));
                    }
                }
            };

            self.in_flight = Some(Box::pin(send_frame(sender, mac, frame)));
            self.in_flight_seq = seq;
        }

        let waker = Waker::noop();
        let mut cx = Context::from_waker(waker);
        if let Poll::Ready((sender, result)) =
            self.in_flight.as_mut().unwrap().as_mut().poll(&mut cx)
        {
            if let Err(err) = result {
                log::error!("EspNow send error (seq {:?}): {err:?}", self.in_flight_seq);
            }
            if let Some(seq) = self.in_flight_seq.take()
                && let Some(pending) = self.pending.get_mut(&seq)
            {
                pending.sent_at = Some(ember_time::now());
            }
            self.sender = Some(sender);
            self.in_flight = None;
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

fn encode_transport(message: &TransportMessage) -> Vec<u8> {
    let envelope = &message.envelopes.base;
    let content = match &message.payload {
        Payload::AclMessage(m) => bit_efficient::encode(m),
        // TODO: implement this.
        Payload::Bytes(_) => unimplemented!(),
    };
    postcard::to_allocvec(&EspNowMessageSer::new(envelope, &content))
        .expect("failed to serialize message into postcard data format")
}

fn decode_message(bytes: &[u8]) -> Option<TransportMessage> {
    postcard::from_bytes::<EspNowMessageDe>(bytes)
        .inspect_err(|_| log::trace!("reliable espnow: skipping unparsable message"))
        .ok()?
        .into_transport()
        .ok()
}

impl<'c> Acc for ReliableEspNowChannel<'c> {
    fn send(
        &mut self,
        address: &Aid,
        message: TransportMessage,
        callbacks: SendCallbacks,
    ) -> Result<(), ()> {
        if self.sender.is_none() && self.in_flight.is_none() {
            log::error!("EspNow channel is not configured for sending messages.");
            return Err(());
        }

        let mac = crate::util::aid_to_mac(address);
        let inner = encode_transport(&message);

        if inner.len() + RELIABLE_FRAME_OVERHEAD > esp_radio::esp_now::ESP_NOW_MAX_DATA_LEN {
            log::error!(
                "message to {mac:02x?} is {} bytes, over ESP-NOW's {}-byte limit, dropping",
                inner.len(),
                esp_radio::esp_now::ESP_NOW_MAX_DATA_LEN
            );
            return Ok(());
        }

        let seq = self.next_seq;
        self.next_seq = self.next_seq.wrapping_add(1);

        self.pending.insert(
            seq,
            PendingSend {
                mac,
                inner: inner.into_boxed_slice(),
                attempts: 1,
                sent_at: None,
                callbacks,
            },
        );
        self.enqueue_data(seq);
        Ok(())
    }

    fn receive(&mut self, environment: &mut Environment) -> Option<TransportMessage> {
        self.retry_due(environment);

        loop {
            self.poll_send();
            let message = self.receiver.as_mut()?.receive()?;
            let bytes = message.data();
            let src_mac = message.info.src_address;

            if !self.reliable_peers.contains(&src_mac) {
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
                    self.handle_ack(seq, environment);
                    continue;
                }
                ReliableFrame::Data { seq, inner } => {
                    self.send_ack(src_mac, seq);
                    if !self.accept(src_mac, seq) {
                        continue;
                    }
                    return decode_message(&inner);
                }
            }
        }
    }
}
