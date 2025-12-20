pub(super) use esp_wifi::esp_now::{EspNowReceiver, EspNowSender};

use esp_wifi::esp_now::{EspNowReceiver as Receiver, EspNowSender as Sender};

use ember_core::agent::aid::Aid;
use ember_core::message::MessageEnvelope;

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
    fn send(&mut self, address: &Aid, message: MessageEnvelope) -> Result<(), ()> {
        let Some(sender) = self.sender.as_mut() else {
            log::error!("EspNow channel is not configured for sending messages.");
            return Err(());
        };

        if let Err(err) = sender
            .send(
                &aid_to_mac(address),
                &postcard::to_allocvec(&EspNowMessageSer(&message))
                    .expect("failed to serialize message into postcard data format"),
            )
            .and_then(|w| w.wait())
        {
            log::error!("EspNow send error: {:?}", err);
        }
        Ok(())
    }

    fn receive(&mut self) -> Option<MessageEnvelope> {
        let message = self.receiver.as_mut().and_then(|r| r.receive())?;
        let envelope = postcard::from_bytes::<EspNowMessageDe>(message.data())
            .inspect_err(|_| {
                log::trace!("Skipping unparsable message.");
            })
            // Assume that if the message could not be parsed, it was not meant for the agent
            // communication system. In the distributed smart home study, devices locate eachother
            // by broadcasting their service. This message would give an error in this case.
            .ok()?
            .into_envelope();
        Some(envelope)
    }
}
