pub(super) use esp_radio::esp_now::{EspNowReceiver, EspNowSender};

use alloc::boxed::Box;
use alloc::collections::VecDeque;
use alloc::string::ToString;
use alloc::vec::Vec;
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll, Waker};

use esp_radio::esp_now::{EspNowError, EspNowReceiver as Receiver, EspNowSender as Sender};

use ember_core::agent::aid::Aid;
use ember_core::message::repr::payload::bit_efficient;
use ember_core::message::{Payload, TransportMessage};

use crate::Acc;
use crate::util::aid_to_mac;

use crate::serde::espnow::de::EspNowMessageDe;
use crate::serde::espnow::ser::EspNowMessageSer;

type InFlight<'c> = Pin<Box<dyn Future<Output = (Sender<'c>, Result<(), EspNowError>)> + 'c>>;

const MAX_QUEUE_LEN: usize = 32;

pub(super) struct EspNowChannel<'c> {
    sender: Option<Sender<'c>>,
    receiver: Option<Receiver<'c>>,
    // Aid is only stored for logging purposes.
    queue: VecDeque<(Option<Aid>, [u8; 6], Vec<u8>)>,
    priority_queue: VecDeque<(Option<Aid>, [u8; 6], Vec<u8>)>,

    in_flight: Option<InFlight<'c>>,
    in_flight_aid: Option<Aid>,
}

impl<'c> EspNowChannel<'c> {
    pub(super) fn new(sender: Option<Sender<'c>>, receiver: Option<Receiver<'c>>) -> Self {
        Self {
            sender,
            receiver,
            queue: VecDeque::new(),
            priority_queue: VecDeque::new(),
            in_flight: None,
            in_flight_aid: None,
        }
    }

    // TODO: Remove priority dirty fix.
    fn enqueue(
        &mut self,
        address: Option<Aid>,
        mac: [u8; 6],
        frame: Vec<u8>,
        priority: bool,
    ) -> Result<(), ()> {
        if self.sender.is_none() && self.in_flight.is_none() {
            log::error!("EspNow channel is not configured for sending messages.");
            return Err(());
        }

        // ESP-NOW's hard per-packet cap; esp_now_send doesn't check it for us.
        if frame.len() > esp_radio::esp_now::ESP_NOW_MAX_DATA_LEN {
            log::error!(
                "message to {:?} is {} bytes, over ESP-NOW's {}-byte limit, dropping",
                address
                    .as_ref()
                    .map_or_else(|| alloc::format!("{mac:?}"), Aid::to_string),
                frame.len(),
                esp_radio::esp_now::ESP_NOW_MAX_DATA_LEN
            );
            return Ok(());
        }

        // Queue instead of sending immediately due to blocking nature of send. Blocking could take
        // a second on esp32 boards.
        if !priority && self.queue.len() >= MAX_QUEUE_LEN {
            // Oldest first: prioritize whatever's most current over a
            // backlog of stale sends - same "eventually consistent, drop
            // rather than block" spirit as the telemetry channels already
            // use elsewhere in this codebase.
            let dropped = self.queue.pop_front();
            log::warn!(
                "EspNow send queue full ({MAX_QUEUE_LEN}), dropping oldest queued send to {:?}",
                dropped.and_then(|(aid, ..)| aid)
            );
        }
        if priority {
            self.priority_queue.push_back((address, mac, frame));
        } else {
            self.queue.push_back((address, mac, frame));
        }
        Ok(())
    }

    fn drain_queue(&mut self) {
        if self.in_flight.is_none() {
            let Some(sender) = self.sender.take() else {
                return;
            };

            let Some((aid, mac, frame)) = self
                .priority_queue
                .pop_front()
                .or_else(|| self.queue.pop_front())
            else {
                self.sender = Some(sender);
                return;
            };
            self.in_flight = Some(Box::pin(send_frame(sender, mac, frame)));
            self.in_flight_aid = aid;
        }

        let waker = Waker::noop();
        let mut cx = Context::from_waker(waker);
        let poll_result = self.in_flight.as_mut().unwrap().as_mut().poll(&mut cx);

        if let Poll::Ready((sender, result)) = poll_result {
            if let Err(err) = result {
                log::error!("EspNow send error to {:?}: {err:?}", self.in_flight_aid);
            }
            self.sender = Some(sender);
            self.in_flight = None;
            self.in_flight_aid = None;
        }
    }

    fn receive_raw(&mut self) -> Option<(Vec<u8>, [u8; 6])> {
        self.drain_queue();
        let message = self.receiver.as_mut()?.receive()?;
        Some((message.data().to_vec(), message.info.src_address))
    }
}

async fn send_frame<'c>(
    mut sender: Sender<'c>,
    mac: [u8; 6],
    frame: Vec<u8>,
) -> (Sender<'c>, Result<(), EspNowError>) {
    let result = sender.send_async(&mac, &frame).await;
    (sender, result)
}

impl<'c> Acc for EspNowChannel<'c> {
    fn send(
        &mut self,
        address: &Aid,
        message: TransportMessage,
        _callbacks: crate::SendCallbacks,
    ) -> Result<(), ()> {
        let envelope = &message.envelopes.base;
        let content = match &message.payload {
            Payload::AclMessage(m) => bit_efficient::encode(m),
            Payload::Bytes(_) => unimplemented!(),
        };

        let frame = postcard::to_allocvec(&EspNowMessageSer::new(envelope, &content))
            .expect("failed to serialize message into postcard data format");

        self.enqueue(Some(address.clone()), aid_to_mac(address), frame, false)
    }

    fn receive(&mut self, _environment: &mut crate::Environment) -> Option<TransportMessage> {
        let (bytes, _src_mac) = self.receive_raw()?;
        postcard::from_bytes::<EspNowMessageDe>(&bytes)
            .inspect_err(|_| {
                log::trace!("Skipping unparsable message.");
            })
            .ok()?
            .into_transport()
            .ok()
    }
}

mod reliable;
pub use reliable::ReliableEspNowChannel;
