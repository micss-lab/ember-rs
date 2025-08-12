use core::ops::Deref;

use alloc::{
    borrow::Cow,
    collections::{BTreeMap, btree_set::BTreeSet},
};
use esp_hal::delay::Delay;
use esp_wifi::esp_now::{EspNowManager, EspNowReceiver, EspNowSender, PeerInfo};
use macaddr::MacAddr6;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
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
    fn new() -> Self {
        Self(BTreeMap::default())
    }

    pub fn discover(
        sender: &mut EspNowSender,
        receiver: &mut EspNowReceiver,
        manager: &mut EspNowManager,
        system: System,
    ) -> Self {
        let mut this = Self::new();
        let mut last_broadcast = esp_hal::time::now();
        let mut others_complete = BTreeSet::new();

        while !this.complete() || others_complete.len() < 2 {
            if (esp_hal::time::now() - last_broadcast).to_secs() >= 3 {
                last_broadcast = esp_hal::time::now();
                broadcast(&mut *sender, system, &this);
                log::debug!(
                    "complete: {}, others: {}",
                    this.complete(),
                    others_complete.len()
                );
            }

            let Some(message) = receiver.receive() else {
                continue;
            };
            let mac = message.info.src_address;
            let info = postcard::from_bytes::<SystemInfo>(message.data())
                .expect("failed to deserialize system info");
            this.set(info.system, mac);
            if info.complete {
                others_complete.insert(info.system);
            }
            this.update(&info.info);
        }

        let delay = Delay::new();
        let mut extra_count = 2;
        while extra_count != 0 {
            broadcast(sender, system, &this);
            extra_count -= 1;
            delay.delay_millis(3000);
        }

        this.0.values().for_each(|p| {
            let _ = manager.add_peer(peer_info(*p)).ok();
        });

        this
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

impl Deref for DiscoveryInfo {
    type Target = BTreeMap<System, [u8; 6]>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
