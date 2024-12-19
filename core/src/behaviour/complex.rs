use alloc::boxed::Box;

use super::{Behaviour, Context};

pub mod parallel;
pub mod sequential;

pub trait ComplexBehaviour<M>
where
    Self: Sized,
{
    fn add_behaviour(&mut self, behaviour: impl Behaviour<Message = M>);

    fn with_behaviour(mut self, behaviour: impl Behaviour<Message = M>) -> Self {
        self.add_behaviour(behaviour);
        self
    }
}

pub(crate) trait BehaviourQueue<M: 'static> {
    fn next(&mut self) -> Option<Box<dyn Behaviour<Message = M>>>;

    fn schedule(&mut self, behaviour: Box<dyn Behaviour<Message = M>>);

    fn is_finished(&self) -> bool;

    fn action(&mut self, ctx: &mut Context<M>) -> bool {
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

impl<T, M> ComplexBehaviour<M> for T
where
    M: 'static,
    T: BehaviourQueue<M>,
{
    fn add_behaviour(&mut self, behaviour: impl Behaviour<Message = M> + 'static) {
        self.schedule(Box::new(behaviour))
    }
}
