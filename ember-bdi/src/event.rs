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
