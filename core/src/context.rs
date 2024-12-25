use alloc::vec::Vec;

pub struct Context<M> {
    pub(crate) container: Option<ContainerContext>,
    pub(crate) agent: Option<AgentContext>,
    pub(crate) messages: Option<Vec<M>>,
}

#[derive(Default)]
pub(crate) struct ContainerContext {
    pub(crate) should_stop: bool,
}

#[derive(Default)]
pub(crate) struct AgentContext {}

impl<M> Default for Context<M> {
    fn default() -> Self {
        Self {
            messages: None,
            container: None,
            agent: None,
        }
    }
}

impl<M> Context<M> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn message_parent(&mut self, message: M) {
        self.messages.get_or_insert_with(Vec::new).push(message);
    }

    pub fn stop_container(&mut self) {
        self.container
            .get_or_insert_with(ContainerContext::new)
            .should_stop = true;
    }

    pub(crate) fn merge<M2>(
        &mut self,
        Context {
            container,
            agent,
            messages: _,
        }: Context<M2>,
    ) {
        if let Some(container) = container {
            self.container
                .get_or_insert_with(ContainerContext::new)
                .merge(container);
        }
        if let Some(agent) = agent {
            self.agent
                .get_or_insert_with(AgentContext::new)
                .merge(agent);
        }
    }
}

impl ContainerContext {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn merge(&mut self, Self { should_stop }: Self) {
        self.should_stop |= should_stop;
    }
}

impl AgentContext {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn merge(&mut self, _other: Self) {}
}
