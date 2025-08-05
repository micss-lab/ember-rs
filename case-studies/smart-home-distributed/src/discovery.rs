use alloc::{
    borrow::Cow,
    collections::{btree_set::BTreeSet, BTreeMap},
};
use esp_hal::delay::Delay;
use esp_wifi::esp_now::{EspNowManager, EspNowReceiver, EspNowSender, PeerInfo};
use macaddr::MacAddr6;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DiscoveryInfo(BTreeMap<System, [u8; 6]>);

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum System {
    PlantMonitoring,
    DoorControl,
    CenterControl,
}

#[derive(Serialize, Deserialize)]
struct SystemInfo<'a> {
    system: System,
    info: Cow<'a, DiscoveryInfo>,
    complete: bool,
}

impl DiscoveryInfo {
    pub fn discover(
        &mut self,
        sender: &mut EspNowSender,
        receiver: &mut EspNowReceiver,
        manager: &mut EspNowManager,
        system: System,
    ) {
        let mut last_broadcast = esp_hal::time::now();
        let mut others_complete = BTreeSet::new();

        while !self.complete() || others_complete.len() < 2 {
            if (esp_hal::time::now() - last_broadcast).to_secs() >= 3 {
                last_broadcast = esp_hal::time::now();
                broadcast(&mut *sender, system, self);
                log::debug!(
                    "complete: {}, others: {}",
                    self.complete(),
                    others_complete.len()
                );
            }

            let Some(message) = receiver.receive() else {
                continue;
            };
            let mac = message.info.src_address;
            let info = postcard::from_bytes::<SystemInfo>(message.data())
                .expect("failed to deserialize system info");
            self.set(info.system, mac);
            if info.complete {
                others_complete.insert(info.system);
            }
            self.update(&info.info);
        }

        let delay = Delay::new();
        let mut extra_count = 2;
        while extra_count != 0 {
            broadcast(sender, system, self);
            extra_count -= 1;
            delay.delay_millis(3000);
        }

        self.0.values().for_each(|p| {
            let _ = manager.add_peer(peer_info(*p)).ok();
        });
    }

    fn complete(&self) -> bool {
        self.0.len() == 3
    }

    fn set(&mut self, system: System, mac: [u8; 6]) {
        self.0
            .entry(system)
            .and_modify(|m| {
                if *m == esp_wifi::esp_now::BROADCAST_ADDRESS {
                    *m = mac;
                }
            })
            .or_insert_with(|| {
                log::debug!(
                    "Found service `{:?}` with mac `{}`",
                    system,
                    MacAddr6::from(mac)
                );
                mac
            });
    }

    fn update(&mut self, info: &DiscoveryInfo) {
        info.0
            .iter()
            .for_each(|(system, mac)| self.set(*system, *mac));
    }
}

fn broadcast(sender: &mut EspNowSender, system: System, current_info: &DiscoveryInfo) {
    log::trace!("Broadcasting...");
    // Broadcast a message with this systems info.
    let complete = current_info.complete();
    sender
        .send(
            &esp_wifi::esp_now::BROADCAST_ADDRESS,
            &postcard::to_allocvec(&SystemInfo {
                system,
                info: Cow::Borrowed(current_info),
                complete,
            })
            .expect("failed to serialize system"),
        )
        .expect("failed to broadcast sytem info")
        .wait()
        .expect("failed to broadcast system info");
}

fn peer_info(mac: [u8; 6]) -> PeerInfo {
    PeerInfo {
        peer_address: mac,
        lmk: None,
        channel: None,
        encrypt: false,
    }
}
