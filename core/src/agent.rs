use alloc::string::{String, ToString};

use crate::behaviour::complex::BehaviourQueue;
use crate::behaviour::parallel::{self, ParallelBehaviourQueue};
use crate::behaviour::{Behaviour, Context};
use crate::container::ContainerAgent;

pub struct Agent<M> {
    pub(crate) name: String,
    behaviours: ParallelBehaviourQueue<M>,
}

impl<M> Agent<M> {
    pub fn new(name: impl ToString) -> Self {
        Self {
            name: name.to_string(),
            behaviours: ParallelBehaviourQueue::new(parallel::Strategy::Never),
        }
    }
}

impl<M: 'static> Agent<M> {
    pub fn with_behaviour(mut self, behaviour: impl Behaviour<Message = M>) -> Self {
        self.add_behaviour(behaviour);
        self
    }

    pub fn add_behaviour(&mut self, behaviour: impl Behaviour<Message = M>) {
        use crate::behaviour::ComplexBehaviour;
        self.behaviours.add_behaviour(behaviour);
    }
}

impl<M: 'static> ContainerAgent for Agent<M> {
    fn update(&mut self, _: &mut Context<()>) {
        let mut context = Context::new();
        self.behaviours.action(&mut context);

        // TODO: Do something with the context here.
    }

    fn get_name(&self) -> String {
        self.name.clone()
    }
}
