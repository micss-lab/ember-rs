use test_log::test;

use no_std_framework_core::behaviour::{
    sequential::{SequentialBehaviour, SequentialBehaviourQueue},
    Context, CyclicBehaviour, OneShotBehaviour,
};
use no_std_framework_core::{Agent, Container};

#[test]
fn sequential() {
    static mut RESULT: u32 = 0;

    struct IncrementMessage;

    struct FirstThirdAndFourth;

    impl OneShotBehaviour for FirstThirdAndFourth {
        type Message = IncrementMessage;

        fn action(&self, ctx: &mut Context<Self::Message>) {
            log::debug!("Running action!");
            ctx.message_parent(IncrementMessage)
        }
    }

    struct Second {
        value: u32,
        /// Track the first time this behaviour is run.
        first: bool,
    }

    impl CyclicBehaviour for Second {
        type Message = IncrementMessage;

        fn action(&mut self, ctx: &mut Context<Self::Message>) {
            self.value -= 1;
            ctx.message_parent(IncrementMessage);
            
            if self.first {
                self.first = false;
                assert_eq!(unsafe {RESULT}, 1);
            }
        }

        fn is_finished(&self) -> bool {
            self.value == 0
        }
    }

    struct Sequential;

    impl SequentialBehaviour for Sequential {
        type Message = ();

        type ChildMessage = IncrementMessage;

        fn initial_behaviours(&self) -> SequentialBehaviourQueue<Self::ChildMessage> {
            SequentialBehaviourQueue::new()
                .with_behaviour(FirstThirdAndFourth)
                .with_behaviour(Second { value: 10, first: true })
                .with_behaviour(FirstThirdAndFourth)
                .with_behaviour(FirstThirdAndFourth)
                .with_behaviour(StopContainer)
        }

        fn handle_child_message(&mut self, _message: Self::ChildMessage) {
            unsafe { RESULT += 1 };
        }
    }

    struct StopContainer;

    impl OneShotBehaviour for StopContainer {
        type Message = IncrementMessage;

        fn action(&self, ctx: &mut Context<Self::Message>) {
            ctx.stop_container();
        }
    }

    let container =
        Container::default().with_agent(Agent::new("parallel-agent").with_behaviour(Sequential));
    container.start().unwrap();
    assert_eq!(unsafe { RESULT }, 13);
}
