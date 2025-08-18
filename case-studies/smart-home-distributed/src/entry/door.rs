use core::ptr::addr_of_mut;

use esp_backtrace as _;

use ember::{Aid, Container};
use esp_hal::{
    clock::CpuClock,
    gpio::{Input, Pull},
    rng::Rng,
    timer::timg::TimerGroup,
    uart::UartRx,
};
use esp_wifi::wifi::{WifiDeviceMode, WifiStaDevice};
use macaddr::MacAddr6;

use home_automation::{lock /* , pir */};

use crate::{
    discovery::{self, System},
    wifi,
};

const HEAP_SIZE: usize = 72 * 1024;

const HOSTNAME: &[u8] = b"esp-smart-home-door-control";

const LOCK_PASSWORD: &[u8] = b"1234";

pub fn main() {
    // Set newline mode to linux line endings.
    esp_println::print!("\x1b[20h");
    esp_println::logger::init_logger_from_env();
    esp_alloc::heap_allocator!(HEAP_SIZE);

    log::info!("Running case study `smart-home-door-control`.");

    let setup_time = esp_hal::time::now();

    let peripherals = esp_hal::init({
        let mut config = esp_hal::Config::default();
        config.cpu_clock = CpuClock::max();
        config
    });
    let mut rng = Rng::new(peripherals.RNG);

    log::trace!("Initializing wifi device.");
    let timg0 = TimerGroup::new(peripherals.TIMG0);
    unsafe { &mut *addr_of_mut!(crate::WIFI_INIT) }
        .set(
            esp_wifi::init(timg0.timer0, rng, peripherals.RADIO_CLK)
                .expect("failed to initialize wifi control."),
        )
        .unwrap();

    let (wifi_device, esp_now_create_token) =
        esp_wifi::esp_now::enable_esp_now_with_wifi(peripherals.WIFI);
    let (wifi_device, mut controller) = esp_wifi::wifi::new_with_mode(
        unsafe { &mut *addr_of_mut!(crate::WIFI_INIT) }
            .get()
            .unwrap(),
        wifi_device,
        esp_wifi::wifi::WifiStaDevice,
    )
    .expect("failed to initialize wifi device");
    let (mut esp_now_manager, mut esp_now_sender, mut esp_now_receiver) =
        esp_wifi::esp_now::EspNow::new_with_wifi(
            unsafe { &mut *addr_of_mut!(crate::WIFI_INIT) }
                .get()
                .unwrap(),
            esp_now_create_token,
        )
        .expect("failed to initialize esp-now")
        .split();

    log::trace!("Setting up network stack.");
    wifi::create_network_stack(wifi_device, rng.random(), HOSTNAME);

    log::trace!("Connecting to access point.");
    wifi::connect_to_access_point(&mut controller);

    // Discover services running on the same network.
    let discovery_info = discovery::DiscoveryInfo::discover(
        &mut esp_now_sender,
        &mut esp_now_receiver,
        &mut esp_now_manager,
        discovery::System::DoorControl,
    );

    log::debug!("Found the following services: {:?}", discovery_info);

    log::debug!(
        "Mac address: {}",
        MacAddr6::from(WifiStaDevice.mac_address())
    );

    // Home automation.
    let unlock_door_switch = Input::new(peripherals.GPIO22, Pull::Up);
    // let pir_sensor = Input::new(peripherals.GPIO18, Pull::Up);

    let serial_input = UartRx::new(peripherals.UART0, Default::default())
        .unwrap()
        .with_rx(peripherals.GPIO3);

    let mut container = Container::default()
        .with_agent(lock::lock_agent(
            LOCK_PASSWORD,
            unlock_door_switch,
            serial_input,
        ))
        // .with_agent(pir::pir_agent(pir_sensor))
        .with_agent_proxy(
            "control",
            Aid::general(
                "control",
                MacAddr6::from(discovery_info[&System::CenterControl]),
            ),
        )
        // .with_agent(control::control_agent(pump_switch, fan_active_led))
        .with_espnow(Some(esp_now_sender), Some(esp_now_receiver));

    log::debug!(
        "Setup time: {} nanoseconds",
        (esp_hal::time::now() - setup_time).to_nanos()
    );

    let mut ticks = 0;
    let mut last_print = esp_hal::time::now();
    loop {
        if (esp_hal::time::now() - last_print).to_secs() >= 1 {
            log::debug!("Loop ticks per second: {}", ticks);
            ticks = 0;
            last_print = esp_hal::time::now();
        }
        ticks += 1;

        if container.poll().unwrap() {
            break;
        }
    }
}
