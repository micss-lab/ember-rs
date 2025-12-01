use esp_backtrace as _;

use esp_hal::{
    clock::CpuClock,
    gpio::{Input, Pull},
    uart::UartRx,
};

use case_study_home_automation::Measurement;

cfg_if::cfg_if! {
    if #[cfg(feature = "ember-based")] {
        use case_study_home_automation::{control, dht22, fan, lock, pir};
        use ember::Container;
    } else {
        use case_study_home_automation::read_chars_from_uart;
    }
}

const HEAP_SIZE: usize = 72 * 1024;

const MEASUREMENTS: [Measurement; 10] = [
    Measurement {
        temperature: 20.0,
        humidity: 50.0,
    },
    Measurement {
        temperature: 22.0,
        humidity: 55.0,
    },
    Measurement {
        temperature: 24.0,
        humidity: 60.0,
    },
    Measurement {
        temperature: 26.0,
        humidity: 65.0,
    },
    Measurement {
        temperature: 28.0,
        humidity: 70.0,
    },
    Measurement {
        temperature: 26.0,
        humidity: 65.0,
    },
    Measurement {
        temperature: 24.0,
        humidity: 60.0,
    },
    Measurement {
        temperature: 22.0,
        humidity: 55.0,
    },
    Measurement {
        temperature: 20.0,
        humidity: 50.0,
    },
    Measurement {
        temperature: 18.0,
        humidity: 45.0,
    },
];

const LOCK_PASSWORD: &[u8] = b"1234";

pub fn main() {
    let peripheral_start = esp_hal::time::now();

    // Set newline mode to linux line endings.
    esp_println::print!("\x1b[20h");
    esp_println::logger::init_logger_from_env();
    esp_alloc::heap_allocator!(HEAP_SIZE);

    log::info!("Running case study `home-automation`.");

    let peripherals = esp_hal::init({
        let mut config = esp_hal::Config::default();
        config.cpu_clock = CpuClock::max();
        config
    });

    let uart_rx = UartRx::new(peripherals.UART1, Default::default())
        .unwrap()
        .with_rx(peripherals.GPIO3);
    let unlock_button = Input::new(peripherals.GPIO22, Pull::Up);
    let pir_pin = Input::new(peripherals.GPIO18, Pull::None);

    log::trace!("Initialized peripherals");
    log::debug!(
        "Peripheral setup: {} ns",
        (esp_hal::time::now() - peripheral_start).to_nanos()
    );

    cfg_if::cfg_if! {
        if #[cfg(feature = "ember-based")] {
            let ember_start = esp_hal::time::now();
            let mut container = Container::default()
                .with_agent(fan::fan_agent())
                .with_agent(dht22::dht22_agent(MEASUREMENTS.into_iter().cycle()))
                .with_agent(lock::lock_agent(LOCK_PASSWORD, unlock_button, uart_rx))
                .with_agent(pir::pir_agent(pir_pin))
                .with_agent(control::control_agent());
            log::debug!(
                "Ember setup: {} ns",
                (esp_hal::time::now() - ember_start).to_nanos()
            );

            let mut ticks = 0;
            let mut last_print = esp_hal::time::now();

            loop {
                let should_stop = container.poll().unwrap();
                if should_stop {
                    break;
                }

                ticks += 1;
                if (esp_hal::time::now() - last_print).to_secs() >= 1 {
                    last_print = esp_hal::time::now();
                    log::debug!("Tps: {}", ticks);
                    ticks = 0;
                }
            }
        } else {
            // This version of the case study is reduced to only what is present in the research
            // paper.

            let mut door_locked = false;
            let mut unlocked_at = esp_hal::time::now();

            let mut object_detected = false;

            let mut serial_rx = uart_rx;
            let mut pir_pin = pir_pin;

            let mut unlock = || {
                use bstr::ByteSlice;

                log::info!("Unlocking door, enter password:");

                let mut password = [0u8; 25];
                read_chars_from_uart(&mut password, &mut serial_rx);

                log::debug!("Password: {}", password.as_bstr());

                if password == LOCK_PASSWORD {
                    log::info!("Password correct, unlocking!");
                    door_locked = false;
                } else {
                    log::debug!("password: {password:?}");
                    log::debug!("set password: {:?}", LOCK_PASSWORD);
                    log::info!("Incorrect password, door remains locked.");
                }
            };

            loop {
                object_detected = pir_pin.is_high();

                if !door_locked && !object_detected && (esp_hal::time::now() - unlocked_at).to_secs() >= 5 {
                    log::info!("Automatically locking door.");
                    *door_locked = true;
                }

                if unlock_button.is_low() && door_locked {
                    log::info!("Unlock button pressed.");
                    unlock();

                    unlocked_at = esp_hal::time::now();
                }
            }
        }
    }
}
