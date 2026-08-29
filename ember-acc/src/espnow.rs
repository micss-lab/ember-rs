pub(super) use esp_radio::esp_now::{EspNowReceiver, EspNowSender};

use esp_radio::esp_now::{EspNowReceiver as Receiver, EspNowSender as Sender};

use ember_core::agent::aid::Aid;
use ember_core::message::repr::payload::bit_efficient;
use ember_core::message::{Payload, TransportMessage};

use crate::Acc;
use crate::util::aid_to_mac;

use crate::serde::espnow::de::EspNowMessageDe;
use crate::serde::espnow::ser::EspNowMessageSer;

pub(super) struct EspNowChannel<'c> {
    sender: Option<Sender<'c>>,
    receiver: Option<Receiver<'c>>,
}

impl<'c> EspNowChannel<'c> {
    pub(super) fn new(sender: Option<Sender<'c>>, receiver: Option<Receiver<'c>>) -> Self {
        Self { sender, receiver }
    }
}

impl<'c> Acc for EspNowChannel<'c> {
    fn send(
        &mut self,
        address: &Aid,
        message: TransportMessage,
        _callbacks: crate::SendCallbacks,
    ) -> Result<(), ()> {
        let Some(sender) = self.sender.as_mut() else {
            log::error!("EspNow channel is not configured for sending messages.");
            return Err(());
        };

        let envelope = &message.envelopes.base;
        let content = match &message.payload {
            Payload::AclMessage(m) => bit_efficient::encode(m),
            Payload::Bytes(_) => unimplemented!(),
        };

        let frame = postcard::to_allocvec(&EspNowMessageSer::new(envelope, &content))
            .expect("failed to serialize message into postcard data format");

        // ESP-NOW's hard per-packet cap; esp_now_send doesn't check it for us.
        if frame.len() > esp_radio::esp_now::ESP_NOW_MAX_DATA_LEN {
            log::error!(
                "message to {address} is {} bytes, over ESP-NOW's {}-byte limit, dropping",
                frame.len(),
                esp_radio::esp_now::ESP_NOW_MAX_DATA_LEN
            );
            return Ok(());
        }

        if let Err(err) = sender
            .send(&aid_to_mac(address), &frame)
            .and_then(|w| w.wait())
        {
            log::error!("EspNow send error to {address}: {:?}", err);
        }
        Ok(())
    }

    fn receive(&mut self, _environment: &mut crate::Environment) -> Option<TransportMessage> {
        let message = self.receiver.as_mut().and_then(|r| r.receive())?;
        postcard::from_bytes::<EspNowMessageDe>(message.data())
            .inspect_err(|_| {
                log::trace!("Skipping unparsable message.");
            })
            // Assume that if the message could not be parsed, it was not meant for the agent
            // communication system. In the distributed smart home study, devices locate eachother
            // by broadcasting their service. This message would give an error in this case.
            .ok()?
            .into_transport()
            .ok()
    }
}
