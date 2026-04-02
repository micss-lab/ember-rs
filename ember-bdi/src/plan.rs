use alloc::boxed::Box;
use alloc::vec::Vec;

use crate::literal::Literal;
use crate::term::Term;

pub struct Plan {
    trigger: TriggeringEvent,
    context: Option<ContextFormula>,
    body: Vec<Formula>,
}

pub struct TriggeringEvent {
    trigger: Trigger,
    event: Literal,
    goal: Option<GoalKind>,
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

pub enum Formula {
    Belief {
        trigger: Trigger,
        belief: Literal,
    },
    Goal {
        kind: GoalKind,
        goal: Literal,
    },
    Action {
        // TODO: find a way to implement this.
        // This will need to interface with rust, so a lookup table might not be the best
        // idea if a function reference is possible.
    },
}
