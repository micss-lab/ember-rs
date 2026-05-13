use alloc::boxed::Box;

use crate::context::Context;
use crate::literal::Literal;
use crate::term::Term;

#[derive(Debug)]
pub struct Plan<A> {
    pub trigger: TriggeringEvent,
    pub context: Option<ContextFormula>,
    pub body: fn(&mut Context) -> Box<[Formula<A>]>,
}

#[derive(Debug, Clone)]
pub struct TriggeringEvent {
    pub trigger: Trigger,
    pub event: Literal,
    pub goal: Option<GoalKind>,
}

#[derive(Debug, Clone, Copy)]
pub enum Trigger {
    Addition,
    Deletion,
}

#[derive(Debug, Clone, Copy)]
pub enum GoalKind {
    Achieve,
    Query,
}

#[derive(Debug)]
pub enum ContextFormula {
    Not(Box<ContextFormula>),
    Logical {
        operator: LogicalOperator,
        operands: Box<[ContextFormula]>,
    },
    Relational {
        operator: RelationalOperator,
        operands: Box<(ArithmeticExpression, ArithmeticExpression)>,
    },
    Literal(Literal),
}

#[derive(Debug, Clone, Copy)]
pub enum LogicalOperator {
    Conjunction,
    Disjunction,
}

#[derive(Debug, Clone, Copy)]
pub enum RelationalOperator {
    Compare {
        operator: CompareOperator,
        equal: bool,
    },
    Unify,
}

#[derive(Debug, Clone, Copy)]
pub enum CompareOperator {
    LessThan,
    GreaterThan,
    EqualTo,
}

#[derive(Debug)]
pub enum ArithmeticExpression {
    Term(Term),
    Operation {
        operator: ArithmeticOperator,
        operands: Box<[ArithmeticExpression]>,
    },
}

#[derive(Debug, Clone, Copy)]
pub enum ArithmeticOperator {
    Sum,
    Min,
    Div,
    Mul,
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
