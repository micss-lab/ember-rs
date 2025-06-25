use alloc::boxed::Box;

use super::{Behaviour, BehaviourId, Context};

pub(crate) trait BehaviourScheduler<E: 'static> {
    fn next(&mut self) -> Option<Box<dyn Behaviour<Event = E>>>;

    fn reschedule(&mut self, behaviour: Box<dyn Behaviour<Event = E>>);

    fn remove(&mut self, id: BehaviourId) -> bool;

    fn block(&mut self, id: BehaviourId) -> bool;

    fn unblock_all(&mut self);

    fn is_finished(&self) -> bool;

    fn action(&mut self, ctx: &mut Context<E>) -> bool {
        if ctx.container.new_messages {
            // Unblock all previously blocked behaviours.
            self.unblock_all();
        }

        let Some(mut behaviour) = self.next() else {
            return self.is_finished();
        };
        let id = behaviour.id();

        let finished = behaviour.action(&mut *ctx);

        // Remove requested behaviours.
        core::mem::take(&mut ctx.local.removed_behaviours)
            .into_iter()
            .for_each(|id| {
                self.remove(id);
            });

        if !finished {
            self.reschedule(behaviour);
        }

        // Block the current behaviour if requested.
        if ctx.local.should_block {
            self.block(id);
        }

        self.is_finished()
    }
}
