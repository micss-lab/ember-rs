use alloc::{collections::BTreeSet, rc::Rc};
use blocking_network_stack::Stack;
use core::{
    cell::{OnceCell, RefCell},
    ptr::addr_of_mut,
};
use esp_wifi::{
    EspWifiController,
    wifi::{WifiController, WifiDevice, WifiStaDevice},
};
use smoltcp::{
    iface::{Interface, SocketSet, SocketStorage},
    phy::Device,
    socket::dhcpv4,
    wire::DhcpOption,
};

use esp_backtrace as _;

use ember::Container;
use esp_hal::{
    analog::adc::{Adc, AdcConfig, Attenuation},
    clock::CpuClock,
    delay::Delay,
    gpio::{Input, Level, Output, Pull},
    rng::Rng,
    timer::timg::TimerGroup,
    uart::UartRx,
};

use home_automation::{fan, lock, pir};
use plant_monitoring::{light, moist, pump};

use case_study_smart_home::{control, temp};

const HEAP_SIZE: usize = 72 * 1024;

const HOSTNAME: &[u8] = b"esp-http-server";

const SSID: Option<&str> = option_env!("CASE_STUDY_SSID");
const AP_PASSWORD: Option<&str> = option_env!("CASE_STUDY_AP_PASSWORD");
const WIFI_CHANNEL: Option<u8> = Some(6);
const WIFI_AP_SCAN_COUNT: u32 = 3;

const SOCKET_COUNT: usize = 10;
static mut SOCKET_STORE: [SocketStorage; SOCKET_COUNT] = [SocketStorage::EMPTY; SOCKET_COUNT];

static mut WIFI_INIT: OnceCell<EspWifiController> = OnceCell::new();

const LOCK_PASSWORD: &[u8] = b"1234";

pub fn main() {
    // Set newline mode to linux line endings.
    esp_println::print!("\x1b[20h");
    esp_println::logger::init_logger_from_env();
    esp_alloc::heap_allocator!(HEAP_SIZE);

    log::info!("Running case study `smart-home`.");

    let peripherals = esp_hal::init({
        let mut config = esp_hal::Config::default();
        config.cpu_clock = CpuClock::max();
        config
    });
    let mut rng = Rng::new(peripherals.RNG);

    log::trace!("Initializing wifi device.");
    let timg0 = TimerGroup::new(peripherals.TIMG0);
    unsafe { &mut *addr_of_mut!(WIFI_INIT) }
        .set(
            esp_wifi::init(timg0.timer0, rng, peripherals.RADIO_CLK)
                .expect("failed to initialize wifi control."),
        )
        .unwrap();

    let (wifi_device, mut controller) = esp_wifi::wifi::new_with_mode(
        unsafe { &mut *addr_of_mut!(WIFI_INIT) }.get().unwrap(),
        peripherals.WIFI,
        esp_wifi::wifi::WifiStaDevice,
    )
    .expect("failed to initialize wifi device");

    log::trace!("Setting up network stack.");
    let mut stack = create_network_stack(wifi_device, rng.random());

    log::trace!("Connecting to access point.");
    connect_to_access_point(&mut controller, &mut stack);

    unsafe { &mut *addr_of_mut!(case_study_smart_home::WIFI_STACK) }
        .set(stack)
        .ok();

    let mut adc_config = AdcConfig::new();

    // Plant monitoring system.
    let ldr_sensor = adc_config.enable_pin(peripherals.GPIO26, Attenuation::_6dB);
    let potentiometer = adc_config.enable_pin(peripherals.GPIO27, Attenuation::_6dB);
    let pump_switch = Input::new(peripherals.GPIO13, Pull::Up);
    let low_light_led = Output::new(peripherals.GPIO14, Level::Low);
    let pump_active_led = Output::new(peripherals.GPIO12, Level::Low);

    // Home automation.
    let unlock_door_switch = Input::new(peripherals.GPIO22, Pull::Up);
    let pir_sensor = Input::new(peripherals.GPIO18, Pull::Up);
    let fan_active_led = Output::new(peripherals.GPIO2, Level::Low);

    let temperature_sensor = adc_config.enable_pin(peripherals.GPIO15, Attenuation::_6dB);

    let adc = Rc::new(RefCell::new(Adc::new(peripherals.ADC2, adc_config)));

    let serial_input = UartRx::new(peripherals.UART1, Default::default()).unwrap();

    Container::default()
        .with_agent(moist::moisture_agent(potentiometer, adc.clone()))
        .with_agent(light::light_agent(ldr_sensor, adc.clone(), low_light_led))
        .with_agent(temp::temperature_agent(temperature_sensor, adc.clone()))
        .with_agent(pump::pump_agent(pump_active_led))
        .with_agent(lock::lock_agent(
            LOCK_PASSWORD,
            unlock_door_switch,
            serial_input,
        ))
        .with_agent(fan::fan_agent())
        .with_agent(pir::pir_agent(pir_sensor))
        .with_agent(control::control_agent(pump_switch, fan_active_led))
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

    let mut scan_networks = || {
        controller
            .scan_n::<6>()
            .expect("failed to scan for networks")
    };
    let (mut found, mut count, mut printed) = (false, 0, BTreeSet::new());
    let delay = Delay::new();
    log::debug!("Found following networks:");
    while !found && count < WIFI_AP_SCAN_COUNT {
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
    loop {
        stack.work();

        if stack.is_iface_up() {
            log::info!("Got ip address: {:?}", stack.get_ip_info().unwrap().ip);
            break;
        }
    }
}
