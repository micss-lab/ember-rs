use alloc::boxed::Box;
use alloc::collections::VecDeque;

use super::{Behaviour, ComplexBehaviour, State};

pub struct ParallelBehaviour<S, PS> {
    state: Option<S>,
    strategy: Strategy,
    finished_behaviours: usize,
    behaviours: VecDeque<Box<dyn Behaviour<ParentState = PS>>>,
}

pub enum Strategy {
    All,
    One,
    N(usize),
    Never,
}

impl<S, PS> ParallelBehaviour<S, PS> {
    pub fn new(state: S, strategy: Strategy) -> Self {
        Self {
            state: Some(state),
            strategy,
            finished_behaviours: 0,
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

impl<S, PS> ParallelBehaviour<S, PS> {
    fn next_behaviour(&mut self) -> Option<Box<dyn Behaviour<ParentState = PS>>> {
        self.behaviours.pop_front()
    }

    fn schedule(&mut self, behaviour: Box<dyn Behaviour<ParentState = PS>>) {
        self.behaviours.push_back(behaviour);
    }

    fn is_finished(&self) -> bool {
        match self.strategy {
            Strategy::All => self.behaviours.is_empty(),
            Strategy::One => self.finished_behaviours == 1,
            Strategy::N(n) => self.finished_behaviours == n,
            Strategy::Never => false,
        }
    }
}

impl<S, P> ComplexBehaviour for ParallelBehaviour<S, State<S, P>> {
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

impl<S> ComplexBehaviour for ParallelBehaviour<S, S> {
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
