extern crate alloc;

use esp_backtrace as _;

use blocking_network_stack::Stack;
use esp_hal::{
    clock::CpuClock,
    peripheral::Peripheral,
    peripherals::{Peripherals, RADIO_CLK, TIMG0, WIFI},
    rng::Rng,
    timer::timg::TimerGroup,
};
use esp_wifi::{
    wifi::{WifiController, WifiDevice, WifiStaDevice},
    EspWifiController,
};
use smoltcp::{
    iface::{Interface, SocketSet, SocketStorage},
    socket::dhcpv4,
    wire::DhcpOption,
};

const HOSTNAME: &'static [u8] = b"esp-http-server";

const HEAP_SIZE: usize = 72 * 1024;

const SOCKET_COUNT: usize = 10;
static mut SOCKET_STORE: [SocketStorage; SOCKET_COUNT] = [SocketStorage::EMPTY; SOCKET_COUNT];

pub(crate) fn main() {
    // Set newline mode to linux line endings.
    esp_println::print!("\x1b[20h");
    esp_println::logger::init_logger_from_env();
    esp_alloc::heap_allocator!(HEAP_SIZE);

    log::info!("Running case study `http-server`.");

    let mut peripherals = esp_hal::init({
        let mut config = esp_hal::Config::default();
        config.cpu_clock = CpuClock::max();
        config
    });
    let mut rng = Rng::new(peripherals.RNG);

    log::trace!("Initializing wifi device.");

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    let wifi_init = esp_wifi::init(timg0.timer0, rng, peripherals.RADIO_CLK)
        .expect("failed to initialize wifi control.");
    let (wifi_device, controller) =
        esp_wifi::wifi::new_with_mode(&wifi_init, peripherals.WIFI, esp_wifi::wifi::WifiStaDevice)
            .expect("failed to initialize wifi device");

    log::info!("Setting up network stack.");

    let stack = create_network_stack(wifi_device, rng.random());

    log::info!("Connecting to access point.");
}

fn create_network_stack<'a>(
    mut wifi: WifiDevice<'a, WifiStaDevice>,
    random: u32,
) -> Stack<'static, WifiDevice<'a, WifiStaDevice>> {
    let mut sockets = SocketSet::new(unsafe { &mut SOCKET_STORE[..] });

    let dhcp_socket = {
        let mut socket = dhcpv4::Socket::new();
        socket.set_outgoing_options(&[DhcpOption {
            // Option `host-name`.
            kind: 12,
            data: HOSTNAME,
        }]);
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
        let iface = Interface::new(
            config,
            &mut wifi,
            Instant::from_micros(esp_hal::time::now().duration_since_epoch().to_micros() as i64),
        );
        iface
    };

    Stack::new(
        iface,
        wifi,
        sockets,
        || esp_hal::time::now().duration_since_epoch().to_millis(),
        random,
    )
}
