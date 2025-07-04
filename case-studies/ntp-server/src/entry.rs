use core::cell::OnceCell;

use esp_backtrace as _;
use esp_hal_embassy as _;

use blocking_network_stack::Stack;
use esp_hal::{clock::CpuClock, rng::Rng, timer::timg::TimerGroup};
use esp_wifi::{
    wifi::{WifiController, WifiDevice, WifiStaDevice},
    EspWifiController,
};
use no_std_framework_core::{Agent, Container};
use smoltcp::{
    iface::{Interface, SocketSet, SocketStorage},
    phy::Device,
    socket::dhcpv4,
    wire::DhcpOption,
};

const HEAP_SIZE: usize = 72 * 1024;

const HOSTNAME: &[u8] = b"esp-ntp-server";

const SSID: Option<&str> = option_env!("CASE_STUDY_SSID");
const AP_PASSWORD: Option<&str> = option_env!("CASE_STUDY_AP_PASSWORD");
const WIFI_CHANNEL: Option<u8> = Some(6);

const SOCKET_COUNT: usize = 10;
static mut SOCKET_STORE: [SocketStorage; SOCKET_COUNT] = [SocketStorage::EMPTY; SOCKET_COUNT];

static mut WIFI_INIT: OnceCell<EspWifiController> = OnceCell::new();

mod ntp;
mod wrapper;

pub(crate) fn main() {
    // Set newline mode to linux line endings.
    esp_println::print!("\x1b[20h");
    esp_println::logger::init_logger_from_env();
    esp_alloc::heap_allocator!(HEAP_SIZE);

    log::info!("Running case study `ntp-server`.");

    let peripherals = esp_hal::init({
        let mut config = esp_hal::Config::default();
        config.cpu_clock = CpuClock::max();
        config
    });
    let mut rng = Rng::new(peripherals.RNG);

    log::trace!("Initializing wifi device.");
    let timg0 = TimerGroup::new(peripherals.TIMG0);
    unsafe {
        WIFI_INIT
            .set(
                esp_wifi::init(timg0.timer0, rng, peripherals.RADIO_CLK)
                    .expect("failed to initialize wifi control."),
            )
            .unwrap();
    }
    let (wifi_device, mut controller) = esp_wifi::wifi::new_with_mode(
        unsafe { WIFI_INIT.get() }.unwrap(),
        peripherals.WIFI,
        esp_wifi::wifi::WifiStaDevice,
    )
    .expect("failed to initialize wifi device");

    log::trace!("Setting up network stack.");
    let mut stack = create_network_stack(wifi_device, rng.random());

    log::trace!("Connecting to access point.");
    connect_to_access_point(&mut controller, &mut stack);

    let server = ntp::Server::new(stack);
    Container::default()
        .with_agent(Agent::new("server").with_behaviour(server))
        .start()
        .unwrap()
}

fn create_network_stack(
    mut wifi: WifiDevice<WifiStaDevice>,
    random: u32,
) -> Stack<'static, WifiDevice<WifiStaDevice>> {
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
        Interface::new(
            config,
            &mut wifi,
            Instant::from_micros(esp_hal::time::now().duration_since_epoch().to_micros() as i64),
        )
    };

    Stack::new(
        iface,
        wifi,
        sockets,
        || esp_hal::time::now().duration_since_epoch().to_millis(),
        random,
    )
}

fn connect_to_access_point(
    controller: &mut WifiController,
    stack: &mut Stack<'static, impl Device>,
) {
    use esp_wifi::wifi::{AuthMethod, ClientConfiguration, Configuration};

    let ssid = SSID.unwrap_or("Wokwi-GUEST");
    let password = AP_PASSWORD.unwrap_or_default();

    let auth_method = if password.is_empty() {
        AuthMethod::None
    } else {
        AuthMethod::WPA2Personal
    };

    let config = ClientConfiguration {
        ssid: ssid.try_into().unwrap(),
        password: password.try_into().unwrap(),
        auth_method,
        channel: WIFI_CHANNEL,
        ..Default::default()
    };

    controller
        .set_configuration(&Configuration::Client(config))
        .expect("failed to set wifi configuration");

    controller.start().expect("failed to start wifi controller");

    log::info!("Scanning for wifi networks.");
    let aps = controller
        .scan_n::<6>()
        .inspect(|aps| {
            log::debug!("Found following networks:");
            for ap in aps.0.iter() {
                log::debug!("- {:?}", ap);
            }
        })
        .expect("failed to scan for networks")
        .0;

    if !aps.into_iter().any(|ap| ap.ssid == ssid) {
        panic!("SSID `{}` not found.", ssid);
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
    loop {
        stack.work();

        if stack.is_iface_up() {
            log::info!("Got ip address: {:?}", stack.get_ip_info().unwrap().ip);
            break;
        }
    }
}
