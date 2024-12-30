use alloc::borrow::Cow;
use alloc::string::{String, ToString};

use crate::behaviour::complex::queue::{BehaviourScheduler, ScheduleStrategy};
use crate::behaviour::parallel::{FinishStrategy, ParallelBehaviourQueue};
use crate::behaviour::IntoBehaviour;
use crate::container::ContainerAgent;
use crate::context::{ContainerContext, Context};

pub struct Agent<M> {
    pub(crate) name: String,
    behaviours: ParallelBehaviourQueue<M>,
}

impl<M: 'static> Agent<M> {
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
        self.behaviours
            .schedule(behaviour.into_behaviour(), ScheduleStrategy::End);
    }
}

impl<M: 'static> ContainerAgent for Agent<M> {
    fn update(&mut self, ctx: &mut ContainerContext) {
        let mut context = Context::new();
        self.behaviours.action(&mut context);

        if let Some(container_ctx) = context.container.take() {
            ctx.merge(container_ctx);
        }
    }

    fn get_name(&self) -> Cow<str> {
        Cow::from(&self.name)
    }
}
