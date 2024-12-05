use alloc::boxed::Box;

pub use self::parallel::ParallelBehaviour;

use super::{Behaviour, State};

pub mod parallel;

trait ComplexBehaviour {
    /// The state that is passed down to the child-behaviours.
    type State;

    /// The state that is passed down to this behaviour from the parent.
    type ParentState;

    /// Returns the next behaviour to be scheduled for execution (if any remain).
    fn next_behaviour(&mut self) -> Option<Box<dyn Behaviour<ParentState = Self::State>>>;

    /// Reschedules the behaviour for execution at a later time.
    fn schedule(&mut self, behaviour: Box<dyn Behaviour<ParentState = Self::State>>);

    /// Returns whether this behaviour has reached the finished state.
    fn is_finished(&self) -> bool;

    /// Constructs the `Self::State` type, using the parent state if needed.
    fn construct_state(&mut self, parent_state: Self::ParentState) -> Self::State;

    /// Updates the state stored in the behaviour with the state returned from a child-behaviour.
    fn update_state(&mut self, state: Self::State) -> Self::ParentState;
}

impl<T, P> Behaviour for T
where
    T: ComplexBehaviour<ParentState = P>,
{
    type ParentState = P;

    fn action(
        &mut self,
        ctx: &mut super::Context,
        parent_state: Self::ParentState,
    ) -> (bool, Self::ParentState) {
        let Some(mut behaviour) = self.next_behaviour() else {
            return (self.is_finished(), parent_state);
        };
        let state = self.construct_state(parent_state);
        let (done, state) = behaviour.action(ctx, state);
        let parent_state = self.update_state(state);
        if !done {
            self.schedule(behaviour);
        }
        (self.is_finished(), parent_state)
    }
}
