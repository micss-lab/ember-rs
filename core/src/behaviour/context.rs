#[derive(Default)]
pub struct Context {
    pub(crate) container: ContainerContext,
    pub(crate) agent: AgentContext,
}

#[derive(Default)]
pub(crate) struct ContainerContext {
    pub(crate) should_stop: bool,
}

#[derive(Default)]
pub(crate) struct AgentContext {}

impl Context {
    pub fn stop(&mut self) {
        self.container.should_stop = true;
    }
}
