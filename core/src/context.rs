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

    pub fn stop_container(&mut self) {
        self.container
            .get_or_insert(ContainerContext::default())
            .should_stop = true;
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
