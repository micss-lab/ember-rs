macro_rules! complex_behaviour_methods {
    () => {
        fn handle_child_message(&mut self, message: Self::ChildMessage) {
            let _ = message;
        }

        fn after_child_action(&mut self, ctx: &mut Context<Self::Message>) {
            let _ = ctx;
        }
    };
}

pub(super) use complex_behaviour_methods;
