use alloc::string::{String, ToString};

use crate::behaviour::{parallel, Behaviour, Context, ParallelBehaviour};

static mut COUNTER: usize = 0;

pub struct Agent {
    name: String,
    behaviours: ParallelBehaviour<(), ()>,
}

impl Agent {
    pub fn new(name: impl ToString) -> Self {
        Self {
            name: name.to_string(),
            behaviours: ParallelBehaviour::new((), parallel::Strategy::Never),
        }
    }

    pub(super) fn update(&mut self) {
        let mut context = Context::default();
        self.behaviours.action(&mut context, ());

        log::info!(
            "Running update `{}` of agent {}!",
            unsafe { COUNTER },
            self.name
        );
        unsafe { COUNTER += 1 };

        // TODO: Do something with the context here.
    }
}
