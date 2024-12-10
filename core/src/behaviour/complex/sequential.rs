use alloc::boxed::Box;
use alloc::collections::VecDeque;

use super::{Behaviour, ComplexBehaviour, State};

pub struct SequentialBehaviour<S, PS> {
    state: Option<S>,
    behaviours: VecDeque<Box<dyn Behaviour<ParentState = PS>>>,
}

impl<S, PS> SequentialBehaviour<S, PS> {
    pub fn new(state: S) -> Self {
        Self {
            state: Some(state),
            behaviours: VecDeque::new(),
        }
    }

    pub fn add_behaviour(&mut self, behaviour: impl Behaviour<ParentState = PS> + 'static) {
        self.behaviours.push_back(Box::new(behaviour));
    }

    pub fn with_behaviour(mut self, behaviour: impl Behaviour<ParentState = PS> + 'static) -> Self {
        self.add_behaviour(behaviour);
        self
    }

    pub fn add_boxed_behaviour(
        &mut self,
        behaviour: Box<dyn Behaviour<ParentState = PS> + 'static>,
    ) {
        self.behaviours.push_back(behaviour);
    }
}

impl<S, PS> SequentialBehaviour<S, PS> {
    fn next_behaviour(&mut self) -> Option<Box<dyn Behaviour<ParentState = PS>>> {
        self.behaviours.pop_front()
    }

    fn schedule(&mut self, behaviour: Box<dyn Behaviour<ParentState = PS>>) {
        self.behaviours.push_front(behaviour);
    }

    fn is_finished(&self) -> bool {
        self.behaviours.is_empty()
    }
}

impl<S, P> ComplexBehaviour for SequentialBehaviour<S, State<S, P>> {
    type State = State<S, P>;
    type ParentState = P;

    fn next_behaviour(&mut self) -> Option<Box<dyn Behaviour<ParentState = Self::State>>> {
        self.next_behaviour()
    }

    fn schedule(&mut self, behaviour: Box<dyn Behaviour<ParentState = Self::State>>) {
        self.schedule(behaviour)
    }

    fn is_finished(&self) -> bool {
        self.is_finished()
    }

    fn construct_state(&mut self, parent_state: Self::ParentState) -> Self::State {
        State {
            root: self
                .state
                .take()
                .expect("value should have been placed back in last iteration"),
            parent: parent_state,
        }
    }

    fn update_state(&mut self, state: Self::State) -> Self::ParentState {
        let (state, parent_state) = state.cut_root();
        self.state.replace(state);
        parent_state
    }
}

impl<S> ComplexBehaviour for SequentialBehaviour<S, S> {
    type State = S;
    type ParentState = ();

    fn next_behaviour(&mut self) -> Option<Box<dyn Behaviour<ParentState = Self::State>>> {
        self.next_behaviour()
    }

    fn schedule(&mut self, behaviour: Box<dyn Behaviour<ParentState = Self::State>>) {
        self.schedule(behaviour);
    }

    fn is_finished(&self) -> bool {
        self.is_finished()
    }

    fn construct_state(&mut self, _: Self::ParentState) -> Self::State {
        self.state
            .take()
            .expect("value to be placed back in the previous iteration")
    }

    fn update_state(&mut self, state: Self::State) -> Self::ParentState {
        self.state.replace(state);
    }
}
