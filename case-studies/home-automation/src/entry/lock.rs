use esp_hal::{
    gpio::{Input, InputPin},
    uart::UartRx,
    Blocking,
};
use no_std_framework_core::{
    behaviour::{Context, CyclicBehaviour, TickerBehaviour},
    Agent,
};

pub fn lock_agent<P>(
    password: &'static [u8],
    button: Input<'static, P>,
    serial_rx: UartRx<'static, Blocking>,
) -> Agent<LockState, ()>
where
    P: InputPin,
{
    Agent::new("lock", LockState::new(password, serial_rx))
        .with_behaviour(UnlockButton::new(button))
        .with_behaviour(AutoLock)
}

pub struct LockState {
    locked: bool,
    password: &'static [u8],
    serial_rx: UartRx<'static, Blocking>,
}

impl LockState {
    fn new(password: &'static [u8], serial_rx: UartRx<'static, Blocking>) -> Self {
        Self {
            locked: true,
            password,
            serial_rx,
        }
    }

    fn unlock(&mut self) {
        use bstr::ByteSlice;

        log::info!("Unlocking door, enter password:");

        let mut password = [0u8; 25];
        let mut read_chars = 0;
        loop {
            let byte = match self.serial_rx.read_byte() {
                Ok(b) => {
                    log::debug!("byte: {}", b);
                    b
                }
                Err(esp_hal::prelude::nb::Error::WouldBlock) => continue,
                Err(e) => panic!("failed to read from console: {:?}", e),
            };

            if byte == b'\n' || byte == b'\r' {
                break;
            }
            password[read_chars] = byte;
            read_chars += 1;
            if read_chars == 25 {
                break;
            }
        }

        log::debug!("Password: {}", password.as_bstr());

        if password[..self.password.len()] == *self.password {
            log::info!("Password correct, unlocking!");
            self.locked = false;
        } else {
            log::debug!("password: {:?}", password);
            log::debug!("set password: {:?}", self.password);
            log::info!("Incorrect password, door remains locked.");
        }
    }

    fn lock(&mut self) {
        self.locked = true;
    }
}

struct UnlockButton<P: 'static> {
    button: Input<'static, P>,
    was_pressed: bool,
}

impl<P: 'static> UnlockButton<P> {
    fn new(button: Input<'static, P>) -> Self {
        Self {
            button,
            was_pressed: false,
        }
    }
}

impl<P> CyclicBehaviour for UnlockButton<P>
where
    P: InputPin,
{
    type AgentState = LockState;

    type Event = ();

    fn action(&mut self, _: &mut Context<Self::Event>, state: &mut Self::AgentState) {
        let is_pressed = self.button.is_low();
        if is_pressed && !self.was_pressed {
            log::info!("Unlock button pressed.");
            state.unlock();
        }
        self.was_pressed = is_pressed;
    }

    fn is_finished(&self) -> bool {
        false
    }
}

struct AutoLock;

impl TickerBehaviour for AutoLock {
    type AgentState = LockState;

    type Event = ();

    fn interval(&self) -> core::time::Duration {
        core::time::Duration::from_secs(5)
    }

    fn action(&mut self, _: &mut Context<Self::Event>, state: &mut Self::AgentState) {
        if !state.locked {
            log::info!("Automatically locking door.");
            state.lock()
        }
    }

    fn is_finished(&self) -> bool {
        false
    }
}
