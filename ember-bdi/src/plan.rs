use alloc::boxed::Box;
use alloc::vec::Vec;

use crate::literal::Literal;
use crate::term::Term;

pub struct Plan<A> {
    pub trigger: TriggeringEvent,
    pub context: Option<ContextFormula>,
    pub body: Vec<Formula<A>>,
}

pub struct TriggeringEvent {
    pub trigger: Trigger,
    pub event: Literal,
    pub goal: Option<GoalKind>,
}

pub enum Trigger {
    Addition,
    Deletion,
}

pub enum GoalKind {
    Achieve,
    Query,
}

pub enum ContextFormula {
    Not(Box<ContextFormula>),
    Logical {
        operator: LogicalOperator,
        operands: Vec<ContextFormula>,
    },
    Relational {
        operator: RelationalOperator,
        operands: Box<(ArithmeticExpression, ArithmeticExpression)>,
    },
    Literal(Literal),
}

pub enum LogicalOperator {
    Conjunction,
    Disjunction,
}

pub enum RelationalOperator {
    Compare {
        operator: CompareOperator,
        equal: bool,
    },
    Unify,
}

pub enum CompareOperator {
    LessThan,
    GreaterThan,
    EqualTo,
}

pub enum ArithmeticExpression {
    Term(Term),
    Operation {
        operator: ArithmeticOperator,
        operands: Vec<ArithmeticExpression>,
    },
}

pub enum ArithmeticOperator {
    Sum,
    Min,
    Div,
    Mul,
}

pub enum Formula<A> {
    Belief { trigger: Trigger, belief: Literal },
    Goal { kind: GoalKind, goal: Literal },
    Action(Action<A>),
}

pub enum Action<A> {
    // TODO: Implement system supported actions.
    // For example, sending messages.
    System(()),
    User(A),
}
