use ember::{
    Agent,
    behaviour::{Context, CyclicBehaviour, TickerBehaviour},
};
use esp_hal::{Blocking, gpio::Input, uart::UartRx};
use ontology::DoorLockAction;

use crate::util::wrap_message;

pub fn lock_agent(
    password: &'static [u8],
    button: Input<'static>,
    serial_rx: UartRx<'static, Blocking>,
) -> Agent<LockState, ()> {
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
            let mut buf = [0u8; 1];
            let byte = match self.serial_rx.read_buffered_bytes(&mut buf) {
                Ok(0) => continue,
                Ok(1) => {
                    let b = buf[0];
                    log::debug!("byte: {b}");
                    b
                }
                Ok(_) => unreachable!("cannot read more bytes than size of buffer"),
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
            log::debug!("password: {password:?}");
            log::debug!("set password: {:?}", self.password);
            log::info!("Incorrect password, door remains locked.");
        }
    }

    fn lock(&mut self) {
        self.locked = true;
    }
}

pub mod ontology {
    use ember::{
        Aid,
        message::{Content, Message, Performative, Receiver},
    };
    use serde::{Deserialize, Serialize};

    pub struct DoorLockOntology;

    #[derive(Serialize, Deserialize)]
    pub enum DoorLockAction {
        Lock,
        Unlock,
    }

    impl DoorLockOntology {
        pub const fn name() -> &'static str {
            "Door-Lock-Ontology"
        }

        pub fn decode_message(message: Message) -> DoorLockAction {
            let Content::Bytes(content) = message.content else {
                panic!("received incorrect content type");
            };
            postcard::from_bytes(&content).expect("failed to parse content")
        }
    }

    impl DoorLockAction {
        pub fn into_message(self) -> Message {
            Message {
                performative: Performative::Inform,
                sender: None,
                receiver: Receiver::Single(Aid::local("control")),
                reply_to: None,
                ontology: Some(DoorLockOntology::name().into()),
                content: Content::Bytes(postcard::to_allocvec(&self).unwrap()),
            }
        }

        pub fn locked(&self) -> bool {
            match self {
                DoorLockAction::Lock => true,
                DoorLockAction::Unlock => false,
            }
        }
    }
}

struct UnlockButton {
    button: Input<'static>,
    was_pressed: bool,
}

impl UnlockButton {
    fn new(button: Input<'static>) -> Self {
        Self {
            button,
            was_pressed: false,
        }
    }
}

impl CyclicBehaviour for UnlockButton {
    type AgentState = LockState;

    type Event = ();

    fn action(&mut self, ctx: &mut Context<Self::Event>, state: &mut Self::AgentState) {
        let is_pressed = self.button.is_low();
        if is_pressed && !self.was_pressed {
            log::info!("Unlock button pressed.");
            state.unlock();

            if !state.locked {
                ctx.send_message(wrap_message(DoorLockAction::Unlock.into_message()));
            }
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

    fn action(&mut self, ctx: &mut Context<Self::Event>, state: &mut Self::AgentState) {
        if !state.locked {
            log::info!("Automatically locking door.");
            state.lock();
            ctx.send_message(wrap_message(DoorLockAction::Lock.into_message()))
        }
    }

    fn is_finished(&self) -> bool {
        false
    }
}
