use alloc::string::{String, ToString};

use crate::behaviour::{parallel, Behaviour, Context, ParallelBehaviour};

pub struct Agent {
    pub(crate) name: String,
    behaviours: ParallelBehaviour<(), ()>,
}

impl Agent {
    pub fn new(name: impl ToString) -> Self {
        Self {
            name: name.to_string(),
            behaviours: ParallelBehaviour::new((), parallel::Strategy::Never),
        }
    }

    pub fn with_behaviour(mut self, behaviour: impl Behaviour<ParentState = ()> + 'static) -> Self {
        self.add_behaviour(behaviour);
        self
    }

    pub fn add_behaviour(&mut self, behaviour: impl Behaviour<ParentState = ()> + 'static) {
        self.behaviours.add_behaviour(behaviour);
    }

    pub(super) fn update(&mut self, context: &mut Context) {
        self.behaviours.action(context, ());

        // TODO: Do something with the context here.
    }
}
