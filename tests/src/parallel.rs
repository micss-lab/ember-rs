use std::{cell::RefCell, rc::Rc};

use no_std_framework_core::behaviour::{
    parallel::{FinishStrategy, ParallelBehaviour, ParallelBehaviourQueue},
    Context, CyclicBehaviour, OneShotBehaviour,
};
use no_std_framework_core::{Agent, Container};

#[test]
fn strategy_all() {
    static mut RESULT: u32 = 0;

    struct IncrementMessage;

    struct FirstAndThird;

    impl OneShotBehaviour for FirstAndThird {
        type Message = IncrementMessage;

        fn action(&self, ctx: &mut Context<Self::Message>) {
            ctx.message_parent(IncrementMessage)
        }
    }

    struct Second {
        value: u32,
    }

    impl CyclicBehaviour for Second {
        type Message = IncrementMessage;

        fn action(&mut self, ctx: &mut Context<Self::Message>) {
            self.value -= 1;
            ctx.message_parent(IncrementMessage);
            if self.is_finished() {
                ctx.stop_container();
            }
        }

        fn is_finished(&self) -> bool {
            self.value == 0
        }
    }

    struct Parallel;

    impl ParallelBehaviour for Parallel {
        type Message = ();

        type ChildMessage = IncrementMessage;

        fn initial_behaviours(&self) -> ParallelBehaviourQueue<Self::ChildMessage> {
            ParallelBehaviourQueue::new(FinishStrategy::All)
                .with_behaviour(FirstAndThird)
                .with_behaviour(Second { value: 10 })
                .with_behaviour(FirstAndThird)
        }

        fn handle_child_message(&mut self, message: Self::ChildMessage) {
            unsafe { RESULT += 1 };
        }
    }

    let container =
        Container::default().with_agent(Agent::new("parallel-agent").with_behaviour(Parallel));
    container.start().unwrap();
    assert!(unsafe { RESULT } == 12);
}
