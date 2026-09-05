use alloc::boxed::Box;
use alloc::vec::Vec;

use ember_core::environment::Environment;
use ember_time::{Duration, Instant};

use ember_collections::SmallMap;

use crate::SendCallbacks;

// 200ms/400ms/800ms/1600ms/3200ms, ~6.2s cumulative worst case before giving up
// (shrunk from a 700ms base, ~21.7s cumulative, to bound real failover/liveness
// detection latency for case studies that react to on_failure).
const RETRY_BASE_INTERVAL: Duration = Duration::millis(200);
const MAX_ATTEMPTS: u32 = 5;
// Messages are stored as rust objects, requiring some math to check payload size is under the max.
pub(super) const RELIABLE_FRAME_OVERHEAD: usize = 8;

#[derive(serde::Serialize, serde::Deserialize)]
pub(super) enum ReliableFrame {
    Data { seq: u32, inner: Vec<u8> },
    Ack { seq: u32 },
}

#[derive(serde::Serialize)]
pub(super) enum ReliableFrameOut<'a> {
    Data { seq: u32, inner: &'a [u8] },
    Ack { seq: u32 },
}

pub(super) struct PendingSend {
    pub(super) mac: [u8; 6],
    pub(super) inner: Box<[u8]>,
    pub(super) attempts: u32,
    pub(super) sent_at: Option<Instant>,
    pub(super) callbacks: SendCallbacks,
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

pub(super) struct ReliableState {
    pub(super) next_seq: u32,
    pub(super) pending: SmallMap<u32, PendingSend>,
    seen: Vec<SeenSender>,
}

impl ReliableState {
    pub(super) fn new() -> Self {
        Self {
            next_seq: 0,
            pending: SmallMap::new(),
            seen: Vec::new(),
        }
    }

    // Split out so tests can drive it without waiting on real wall-clock time.
    pub(super) fn retry_due_at(&mut self, now: Instant, environment: &mut Environment) -> Vec<u32> {
        let due: Vec<u32> = self
            .pending
            .iter()
            .filter(|(_, entry)| entry.attempts < MAX_ATTEMPTS)
            .filter_map(|(seq, entry)| {
                let sent_at = entry.sent_at?;
                (now - sent_at >= entry.retry_after()).then_some(*seq)
            })
            .collect();

        for seq in &due {
            let Some(entry) = self.pending.get_mut(seq) else {
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
        }

        let mut failures = Vec::new();
        self.pending.retain(|seq, entry| {
            let exhausted = entry.attempts >= MAX_ATTEMPTS;
            if exhausted {
                log::warn!(
                    "reliable espnow: giving up on seq {seq} after {MAX_ATTEMPTS} attempts, no ack received"
                );
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

        due
    }

    // Split out so tests can drive it without a real ack frame arriving over
    // the radio.
    pub(super) fn handle_ack(&mut self, seq: u32, environment: &mut Environment) {
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

    /// Message deduplication ensuring a message arrives at most a single time.
    pub(super) fn accept(&mut self, mac: [u8; 6], seq: u32) -> bool {
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

    pub(super) fn drain(self, environment: &mut Environment) {
        for (_, mut entry) in self.pending {
            if let Some(on_failure) = entry.callbacks.on_failure.take() {
                on_failure(environment);
            }
            if let Some(on_complete) = entry.callbacks.on_complete.take() {
                on_complete(environment);
            }
        }
    }
}
