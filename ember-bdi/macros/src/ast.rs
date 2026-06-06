#[derive(Debug)]
pub(crate) struct Program {
    pub(crate) beliefs: Box<[Belief]>,
    pub(crate) initial: Box<[InitGoal]>,
    pub(crate) plans: Box<[Plan]>,
}

#[derive(Debug)]
pub(crate) struct Belief(pub(crate) Literal);

#[derive(Debug)]
pub(crate) struct Literal {
    pub(crate) negated: bool,
    pub(crate) formula: AtomicFormula,
}

#[derive(Debug)]
pub(crate) struct AtomicFormula {
    pub(crate) functor: AtomOrVar,
    pub(crate) arguments: Box<[Term]>,
}

#[derive(Debug)]
pub(crate) enum AtomOrVar {
    Var(Variable),
    Atom(Atom),
}

#[derive(Debug)]
pub(crate) struct Variable(pub(crate) String);

#[derive(Debug)]
pub(crate) struct Atom(pub(crate) String);

#[derive(Debug)]
pub(crate) enum Term {
    Literal(Literal),
    Var(Variable),
    Number(f32),
    String(String),
}

#[derive(Debug)]
pub(crate) struct InitGoal(pub(crate) Literal);

#[derive(Debug)]
pub(crate) struct Plan {
    pub(crate) event: TriggeringEvent,
    pub(crate) context: Option<Context>,
    pub(crate) body: Body,
}

#[derive(Debug)]
pub(crate) struct TriggeringEvent {
    pub(crate) trigger: Trigger,
    pub(crate) goal: Option<Goal>,
    pub(crate) event: Literal,
}

#[derive(Debug)]
pub(crate) enum Trigger {
    Addition,
    Deletion,
}

#[derive(Debug)]
pub(crate) enum Goal {
    Achieve,
    Query,
}

#[derive(Debug)]
pub(crate) struct Context(pub(crate) LogicalExpression);

#[derive(Debug)]
pub(crate) enum LogicalExpression {
    Simple(SimpleLogicalExpression),
    Not(Box<LogicalExpression>),
    And(Box<(SimpleLogicalExpression, LogicalExpression)>),
    Or(Box<(SimpleLogicalExpression, LogicalExpression)>),
}

#[derive(Debug)]
pub(crate) enum SimpleLogicalExpression {
    Literal(Literal),
    Rel(RelationalExpression),
}
#[derive(Debug)]
pub(crate) struct RelationalExpression {
    pub(crate) operator: RelationalOperator,
    pub(crate) operands: (RelationalTerm, RelationalTerm),
}

#[derive(Debug)]
pub(crate) enum RelationalOperator {
    Smaller,
    Larger,
    SmallerEq,
    LargerEq,
    Equal,
    NotEqual,
    Unify,
}

#[derive(Debug)]
pub(crate) enum RelationalTerm {
    Literal(Literal),
    Arithm(ArithmeticExpression),
}

#[derive(Debug)]
pub(crate) struct ArithmeticExpression {
    pub(crate) operator: PlusMin,
    pub(crate) operands: (ArithmeticTerm, Option<Box<ArithmeticExpression>>),
}

#[derive(Debug)]
pub(crate) enum PlusMin {
    Plus,
    Min,
}

#[derive(Debug)]
pub(crate) struct ArithmeticTerm {
    pub(crate) operator: DivMul,
    pub(crate) operands: (ArithmeticFactor, Option<Box<ArithmeticTerm>>),
}

#[derive(Debug)]
pub(crate) enum DivMul {
    Division,
    Multiplication,
}

#[derive(Debug)]
pub(crate) enum ArithmeticFactor {
    Number(f32),
    Var(Variable),
    Neg(Box<ArithmeticFactor>),
    Group(Box<ArithmeticExpression>),
}

#[derive(Debug)]
pub(crate) struct Body(pub(crate) Box<[BodyFormula]>);

#[derive(Debug)]
pub(crate) enum BodyFormula {
    BeliefOrGoal {
        trigger: BodyFormulaTrigger,
        literal: Literal,
    },
    Atomic(AtomicFormula),
}

#[derive(Debug)]
pub(crate) enum BodyFormulaTrigger {
    Add,
    Remove,
    Update,
    Achieve,
    Query,
}
