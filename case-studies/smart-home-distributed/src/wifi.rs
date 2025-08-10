use core::ptr::addr_of_mut;

use alloc::{boxed::Box, collections::BTreeSet};

use blocking_network_stack::Stack;
use esp_hal::delay::Delay;
use esp_wifi::wifi::{WifiController, WifiDevice, WifiStaDevice};
use smoltcp::{
    iface::{Interface, SocketSet},
    socket::dhcpv4,
    wire::DhcpOption,
};

pub fn create_network_stack(
    mut wifi: WifiDevice<'static, WifiStaDevice>,
    random: u32,
    hostname: &'static [u8],
) {
    let mut sockets = SocketSet::new(unsafe { &mut crate::SOCKET_STORE[..] });

    let dhcp_socket = {
        let mut socket = dhcpv4::Socket::new();
        let options = Box::new([DhcpOption {
            // Option `host-name`.
            kind: 12,
            data: hostname,
        }]);
        socket.set_outgoing_options(Box::leak(options));
        socket
    };

    sockets.add(dhcp_socket);

    let iface = {
        use esp_wifi::wifi::WifiDeviceMode;
        use smoltcp::{
            iface::Config,
            time::Instant,
            wire::{EthernetAddress, HardwareAddress},
        };

        let config = {
            let mac = WifiStaDevice.mac_address();
            let hw_address = HardwareAddress::Ethernet(EthernetAddress::from_bytes(&mac));
            Config::new(hw_address)
        };
        Interface::new(
            config,
            &mut wifi,
            Instant::from_micros(esp_hal::time::now().duration_since_epoch().to_micros() as i64),
        )
    };

    unsafe { &mut *addr_of_mut!(crate::WIFI_STACK) }
        .set(Stack::new(
            iface,
            wifi,
            sockets,
            || esp_hal::time::now().duration_since_epoch().to_millis(),
            random,
        ))
        .ok()
        .expect("cannot create stack more than once");
}

pub fn connect_to_access_point(controller: &mut WifiController) {
    use esp_wifi::wifi::{AuthMethod, ClientConfiguration, Configuration};

    let ssid = crate::SSID.unwrap_or("Wokwi-GUEST");
    let password = crate::AP_PASSWORD.unwrap_or_default();

    let auth_method = if password.is_empty() {
        AuthMethod::None
    } else {
        AuthMethod::WPA2Personal
    };

    let config = ClientConfiguration {
        ssid: ssid.try_into().unwrap(),
        password: password.try_into().unwrap(),
        auth_method,
        channel: crate::WIFI_CHANNEL,
        ..Default::default()
    };

    controller
        .set_configuration(&Configuration::Client(config))
        .expect("failed to set wifi configuration");

    controller.start().expect("failed to start wifi controller");

    log::info!("Scanning for wifi networks.");

    let mut scan_networks = || {
        controller
            .scan_n::<6>()
            .expect("failed to scan for networks")
    };
    let (mut found, mut count, mut printed) = (false, 0, BTreeSet::new());
    let delay = Delay::new();
    log::debug!("Found following networks:");
    while !found && count < crate::WIFI_AP_SCAN_COUNT {
        log::trace!("Scanning");
        count += 1;
        found = scan_networks()
            .0
            .into_iter()
            .inspect(|ap| {
                if printed.insert(ap.ssid.clone()) {
                    log::debug!("- {:?}", ap);
                }
            })
            .any(|ap| ap.ssid == ssid);
        delay.delay_millis(200);
    }

    if !found {
        log::warn!("SSID `{}` not found, attempting to connect anyway.", ssid);
    }

    log::info!("Connecting to access point `{}`", ssid);
    controller
        .connect()
        .expect("failed to connect to access point");

    loop {
        match controller.is_connected() {
            Ok(true) => {
                log::info!("Connected!");
                break;
            }
            Err(err) => panic!("failed to connect to access point: {:?}", err),
            _ => {
                continue;
            }
        }
    }

    log::trace!("Waiting for an ip address.");
    let stack = unsafe { &mut *addr_of_mut!(crate::WIFI_STACK) }
        .get_mut()
        .expect("wifi stack should be created before calling this function");
    loop {
        stack.work();

        if stack.is_iface_up() {
            log::info!("Got ip address: {:?}", stack.get_ip_info().unwrap().ip);
            break;
        }
    }
}
