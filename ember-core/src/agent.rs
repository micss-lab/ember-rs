use alloc::borrow::Cow;

use crate::context::ContainerContext;

pub mod aid;

pub trait AgentLike {
    fn update(&mut self, context: &mut ContainerContext) -> bool;

    fn get_name(&self) -> Cow<str>;
}
