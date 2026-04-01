pub struct BeliefBase {
    beliefs: Vec<Belief>,
}

pub struct Belief {
    negated: bool,
    atom: Term,
    rule: Option<Formula>,
}

pub enum Term {
    Number(f32),
    String(BString),
    Variable(String),
    List(Vec<Term>),
    Structure(Structure),
}

pub struct Atom(String);

pub struct Structure {
    functor: Atom,
    arguments: Vec<Term>,
}

pub enum Formula {
    Logical {
        operator: LogicalOperator,
        operands: Vec<Formula>,
    },
    Relational {
        operator: RelationalOperator,
        operands: Box<(Formula, Formula)>,
    },
    Arithmetic(ArithmeticExpression),
    Term(Term),
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
    Factor(Factor),
    Operation(ArithmeticOperation),
}

pub struct ArithmeticOperation {
    operator: ArithmeticOperator,
    operands: Vec<ArithmeticExpression>,
}

pub enum Factor {
    Number(f32),
    Variable(String),
}
