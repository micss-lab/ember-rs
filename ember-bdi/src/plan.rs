use alloc::boxed::Box;

use crate::literal::Literal;

pub use crate::event::{GoalKind, Trigger, TriggeringEvent};
pub use crate::knowledge::query::formula::*;

pub mod library;
pub mod selection;
pub mod selector;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Plan<A> {
    pub trigger: TriggeringEvent,
    pub context: Option<QueryFormula>,
    pub body: Box<[Formula<A>]>,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Formula<A> {
    Belief { trigger: Trigger, belief: Literal },
    Goal { kind: GoalKind, goal: Literal },
    Action(Action<A>),
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Action<A> {
    // TODO: Implement system supported actions.
    // For example, sending messages.
    System(()),
    User(A),
}
