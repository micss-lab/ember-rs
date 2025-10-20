use alloc::rc::Rc;
use core::cell::RefCell;
use esp_wifi::wifi::{WifiDeviceMode, WifiStaDevice};

use esp_backtrace as _;

use ember::{Aid, Container};
use esp_hal::{
    analog::adc::{Adc, AdcConfig, Attenuation},
    clock::CpuClock,
    gpio::{Input, Level, Output, Pull},
    rng::Rng,
    timer::timg::TimerGroup,
};
use macaddr::MacAddr6;

use home_automation::fan;

use crate::{
    control,
    discovery::{self, System},
    temp, wifi,
};

const HEAP_SIZE: usize = 72 * 1024;

const HOSTNAME: &[u8] = b"esp-smart-home-control-panel";

pub fn main() {
    // Set newline mode to linux line endings.
    esp_println::print!("\x1b[20h");
    esp_println::logger::init_logger_from_env();
    esp_alloc::heap_allocator!(HEAP_SIZE);

    log::info!("Running case study `smart-home-control-panel`.");

    let setup_time = esp_hal::time::now();

    let peripherals = esp_hal::init({
        let mut config = esp_hal::Config::default();
        config.cpu_clock = CpuClock::max();
        config
    });
    let mut rng = Rng::new(peripherals.RNG);

    log::trace!("Initializing wifi device.");
    let timg0 = TimerGroup::new(peripherals.TIMG0);
    let wifi_init = esp_wifi::init(timg0.timer0, rng, peripherals.RADIO_CLK)
        .expect("failed to initialize wifi control.");

    let (wifi_device, esp_now_create_token) =
        esp_wifi::esp_now::enable_esp_now_with_wifi(peripherals.WIFI);
    let (wifi_device, mut controller) =
        esp_wifi::wifi::new_with_mode(&wifi_init, wifi_device, esp_wifi::wifi::WifiStaDevice)
            .expect("failed to initialize wifi device");
    let (mut esp_now_manager, mut esp_now_sender, mut esp_now_receiver) =
        esp_wifi::esp_now::EspNow::new_with_wifi(&wifi_init, esp_now_create_token)
            .expect("failed to initialize esp-now")
            .split();

    log::trace!("Setting up network stack.");
    let stack = wifi::create_network_stack(wifi_device, rng.random(), HOSTNAME);

    log::trace!("Connecting to access point.");
    wifi::connect_to_access_point(&stack, &mut controller);

    // Discover services running on the same network.
    let discovery_info = discovery::DiscoveryInfo::discover(
        &mut esp_now_sender,
        &mut esp_now_receiver,
        &mut esp_now_manager,
        discovery::System::CenterControl,
    );

    log::debug!("Found the following services: {:?}", discovery_info);

    log::debug!(
        "Mac address: {}",
        MacAddr6::from(WifiStaDevice.mac_address())
    );

    let mut adc_config = AdcConfig::new();

    let pump_switch = Input::new(peripherals.GPIO5, Pull::Up);
    let fan_active_led = Output::new(peripherals.GPIO18, Level::Low);
    let temperature_sensor = adc_config.enable_pin(peripherals.GPIO34, Attenuation::_6dB);

    let adc = Rc::new(RefCell::new(Adc::new(peripherals.ADC1, adc_config)));

    let mut container = Container::default()
        .with_agent(temp::temperature_agent(temperature_sensor, adc.clone()))
        .with_agent(fan::fan_agent())
        .with_agent(control::control_agent(&stack, pump_switch, fan_active_led))
        .with_agent_proxy(
            "pump",
            Aid::general(
                "pump",
                MacAddr6::from(discovery_info[&System::PlantMonitoring]),
            ),
        )
        .with_espnow(Some(esp_now_sender), Some(esp_now_receiver));

    log::debug!(
        "Setup time: {} nanoseconds",
        (esp_hal::time::now() - setup_time).to_nanos(),
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
