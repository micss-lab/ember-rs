macro_rules! complex_behaviour_methods {
    () => {
        fn handle_child_event(&mut self, message: Self::ChildEvent) {
            let _ = message;
        }

        fn after_child_action(&mut self, ctx: &mut Context<Self::Event>) {
            let _ = ctx;
        }
    };
}

macro_rules! complex_action_impl {
    () => {
        fn action(&mut self, ctx: &mut Context<Self::Event>) -> bool {
            let mut context = Context::from_upper(&mut *ctx);

            // 1. Execute next scheduled behaviour.
            self.queue.action(&mut context);

            // 2. Handle events the behaviour produced.
            while let Some(event) = context.local.events.pop() {
                self.kind.0.handle_child_event(event);
            }

            // 3. Update the parent context.
            ctx.merge(context);

            // 4. Run user defined actions for this complex behaviour.
            self.kind.0.after_child_action(ctx);

            self.queue.is_finished()
        }
    };
}

pub(super) use {complex_action_impl, complex_behaviour_methods};
