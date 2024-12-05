use std::{cell::RefCell, rc::Rc};

use no_std_framework_core::behaviour::{
    parallel, CyclicBehaviour, OneShotBehaviour, ParallelBehaviour, SimpleBehaviourState, State,
};
use no_std_framework_core::{Agent, Container};

#[test]
fn strategy_all() {
    #[derive(Debug)]
    struct SequentialStateInner {
        value: u32,
    }

    struct CyclicState(u32);

    impl SimpleBehaviourState for CyclicState {
        fn finished(&self) -> bool {
            self.0 == 0
        }
    }

    type SequentialState = Rc<RefCell<SequentialStateInner>>;

    let state = Rc::new(RefCell::new(SequentialStateInner { value: 0 }));
    let container = Container::default().with_agent(
        Agent::new("parallel-agent")
            .with_behaviour(OneShotBehaviour::new(|_, _| {
                println!("Some value");
            }))
            .with_behaviour(
                ParallelBehaviour::new(state.clone(), parallel::Strategy::All)
                    .with_behaviour(OneShotBehaviour::new(|_, state: SequentialState| {
                        println!("I hope this will work...");
                        state.borrow_mut().value += 1;
                        state
                    }))
                    .with_behaviour(CyclicBehaviour::new(
                        CyclicState(10),
                        |ctx, mut state: State<CyclicState, SequentialState>| {
                            state.0 -= 1;
                            state.parent().borrow_mut().value += 1;
                            if state.finished() {
                                ctx.stop();
                            }
                            state
                        },
                    ))
                    .with_behaviour(OneShotBehaviour::new(|_, state: SequentialState| {
                        println!("foo");
                        state.borrow_mut().value += 1;
                        state
                    })),
            ),
    );
    container.start().unwrap();
    assert!(state.borrow_mut().value == 12);
}
