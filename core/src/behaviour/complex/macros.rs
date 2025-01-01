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

macro_rules! complex_action_impl {
    () => {
        fn action(&mut self, ctx: &mut Context<Self::Message>) -> bool {
            let mut context = Context::new();

            // 1. Execute next scheduled behaviour.
            self.queue.action(&mut context);

            // 2. Handle messages the behaviour produced.
            while let Some(message) = context.local.messages.pop() {
                self.kind.0.handle_child_message(message);
            }

            // 4. Run user defined actions for this complex behaviour.
            ctx.merge(context);
            self.kind.0.after_child_action(ctx);

            self.queue.is_finished()
        }
    };
}

pub(super) use {complex_action_impl, complex_behaviour_methods};
