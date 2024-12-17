use alloc::string::{String, ToString};

use crate::behaviour::complex::BehaviourQueue;
use crate::behaviour::parallel::{self, ParallelBehaviour};
use crate::behaviour::{Behaviour, Context};

pub struct Agent {
    pub(crate) name: String,
    behaviours: ParallelBehaviour,
}

impl Agent {
    pub fn new(name: impl ToString) -> Self {
        Self {
            name: name.to_string(),
            behaviours: ParallelBehaviour::new(parallel::Strategy::Never),
        }
    }

    pub fn with_behaviour(mut self, behaviour: Behaviour) -> Self {
        self.add_behaviour(behaviour);
        self
    }

    pub fn add_behaviour(&mut self, behaviour: Behaviour) {
        use crate::behaviour::ComplexBehaviour;
        self.behaviours.add_behaviour(behaviour);
    }

    pub(super) fn update(&mut self, context: &mut Context) {
        self.behaviours.action(context);

        // TODO: Do something with the context here.
    }
}
