use crate::intention::IntentionId;
use crate::literal::Literal;

pub(crate) mod queue;
pub mod selector;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct TriggeringEvent {
    pub trigger: Trigger,
    pub event: Literal,
    pub goal: Option<GoalKind>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Trigger {
    Addition,
    Deletion,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum GoalKind {
    Achieve,
    Query,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum EventSource {
    /// The event is fired from the execution of an internal intention.
    Internal(IntentionId),
    /// The event is generated from an external interaction.
    External,
}
