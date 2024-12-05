use alloc::boxed::Box;

use super::{Behaviour, Context, SimpleBehaviourState, State};

pub struct CyclicBehaviour<S: SimpleBehaviourState, P> {
    state: Option<S>,
    #[allow(clippy::type_complexity)]
    action_impl: Box<dyn FnMut(&mut Context, State<S, P>) -> State<S, P> + Send + 'static>,
}

impl<S: SimpleBehaviourState, P> CyclicBehaviour<S, P> {
    pub fn new(
        state: S,
        action: impl FnMut(&mut Context, State<S, P>) -> State<S, P> + Send + 'static,
    ) -> Self {
        Self {
            state: Some(state),
            action_impl: Box::new(action),
        }
    }
}

impl<S: SimpleBehaviourState, P> Behaviour for CyclicBehaviour<S, P> {
    type ParentState = P;

    fn action(
        &mut self,
        ctx: &mut Context,
        parent_state: Self::ParentState,
    ) -> (bool, Self::ParentState) {
        let state = State {
            root: self
                .state
                .take()
                .expect("value should have been placed back in last iteration"),
            parent: parent_state,
        };
        let (state, parent_state) = (self.action_impl)(ctx, state).cut_root();
        let result = state.finished();
        self.state.replace(state);
        (result, parent_state)
    }
}
