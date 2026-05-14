use alloc::boxed::Box;

use derive_where::derive_where;

use crate::context::Context;
use crate::literal::Literal;

pub use crate::knowledge::query::formula::*;

pub mod selector;
pub mod store;

#[derive(Debug)]
#[derive_where(PartialOrd, Ord, PartialEq, Eq)]
pub struct Plan<A> {
    pub trigger: TriggeringEvent,
    pub context: Option<QueryFormula>,
    pub body: fn(&mut Context) -> Box<[Formula<A>]>,
}

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

#[derive(Debug)]
pub enum Formula<A> {
    Belief { trigger: Trigger, belief: Literal },
    Goal { kind: GoalKind, goal: Literal },
    Action(Action<A>),
}

#[derive(Debug)]
pub enum Action<A> {
    // TODO: Implement system supported actions.
    // For example, sending messages.
    System(()),
    User(A),
}
