use alloc::vec::Vec;

pub struct Context<M> {
    pub(crate) container: ContainerContext,
    #[allow(unused)]
    pub(crate) agent: AgentContext,
    pub(crate) messages: Vec<M>,
}

#[derive(Default)]
pub(crate) struct ContainerContext {
    pub(crate) should_stop: bool,
}

#[derive(Default)]
pub(crate) struct AgentContext {}

impl<M> Context<M> {
    pub fn new() -> Self {
        Self {
            container: ContainerContext::default(),
            agent: AgentContext::default(),
            messages: Vec::new(),
        }
    }

    pub fn stop(&mut self) {
        self.container.should_stop = true;
    }
}
