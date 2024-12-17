use super::{Behaviour, Context};

pub mod parallel;
pub mod sequential;

pub trait ComplexBehaviour
where
    Self: Sized,
{
    fn add_behaviour(&mut self, behaviour: Behaviour);

    fn with_behaviour(mut self, behaviour: Behaviour) -> Self {
        self.add_behaviour(behaviour);
        self
    }
}

pub(crate) trait BehaviourQueue {
    fn next(&mut self) -> Option<Behaviour>;

    fn schedule(&mut self, behaviour: Behaviour);

    fn is_finished(&self) -> bool;

    fn action(&mut self, ctx: &mut Context) -> bool {
        let Some(mut behaviour) = self.next() else {
            return self.is_finished();
        };
        let finished = behaviour.action(ctx);
        if !finished {
            self.schedule(behaviour);
        }
        self.is_finished()
    }
}

impl<T> ComplexBehaviour for T
where
    T: BehaviourQueue,
{
    fn add_behaviour(&mut self, behaviour: Behaviour) {
        self.schedule(behaviour);
    }
}
