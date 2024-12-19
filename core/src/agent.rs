use alloc::string::{String, ToString};

use crate::behaviour::complex::BehaviourQueue;
use crate::behaviour::parallel::{FinishStrategy, ParallelBehaviourQueue};
use crate::behaviour::{Context, IntoBehaviour};
use crate::container::ContainerAgent;

pub struct Agent<M> {
    pub(crate) name: String,
    behaviours: ParallelBehaviourQueue<M>,
}

impl<M> Agent<M> {
    pub fn new(name: impl ToString) -> Self {
        Self {
            name: name.to_string(),
            behaviours: ParallelBehaviourQueue::new(FinishStrategy::Never),
        }
    }
}

impl<M: 'static> Agent<M> {
    pub fn with_behaviour<K>(mut self, behaviour: impl IntoBehaviour<K, Message = M>) -> Self {
        self.add_behaviour(behaviour);
        self
    }

    pub fn add_behaviour<K>(&mut self, behaviour: impl IntoBehaviour<K, Message = M>) {
        self.behaviours.schedule(behaviour.into_behaviour());
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
