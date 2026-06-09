use crate::action::BuiltinAction;

#[derive(Debug, Clone)]
pub(crate) struct Spanned<T> {
    pub(crate) node: T,
    pub(crate) span: proc_macro2::Span,
}

impl<T> core::ops::Deref for Spanned<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.node
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Program {
    pub(crate) beliefs: Box<[Spanned<Belief>]>,
    pub(crate) goals: Box<[Spanned<Goal>]>,
    pub(crate) plans: Box<[Spanned<Plan>]>,
}

#[derive(Debug, Clone)]
pub(crate) struct Belief(pub(crate) Literal);

#[derive(Debug, Clone)]
pub(crate) struct Literal {
    pub(crate) negated: bool,
    pub(crate) formula: Spanned<AtomicFormula>,
}

#[derive(Debug, Clone)]
pub(crate) struct AtomicFormula {
    pub(crate) functor: Atom,
    pub(crate) arguments: Option<Box<[Term]>>,
}

#[derive(Debug, Clone)]
pub(crate) struct Variable(pub(crate) String);

#[derive(Debug, Clone)]
pub(crate) struct Atom(pub(crate) String);

#[derive(Debug, Clone)]
pub(crate) enum Term {
    Literal(Literal),
    Variable(Variable),
    Number(f32),
    String(String),
}

#[derive(Debug, Clone)]
pub(crate) struct Goal(pub(crate) Literal);

#[derive(Debug, Clone)]
pub(crate) struct Plan {
    pub(crate) event: TriggeringEvent,
    pub(crate) context: Option<Context>,
    pub(crate) body: Body,
}

#[derive(Debug, Clone)]
pub(crate) struct TriggeringEvent {
    pub(crate) trigger: Trigger,
    pub(crate) goal: Option<EventGoal>,
    pub(crate) event: Literal,
}

#[derive(Debug, Clone)]
pub(crate) enum Trigger {
    Addition,
    Deletion,
}

#[derive(Debug, Clone)]
pub(crate) enum EventGoal {
    Achieve,
    Query,
}

#[derive(Debug, Clone)]
pub(crate) struct Context(pub(crate) LogicalExpression);

#[derive(Debug, Clone)]
pub(crate) enum LogicalExpression {
    Simple(SimpleLogicalExpression),
    And((SimpleLogicalExpression, Box<LogicalExpression>)),
    Or((SimpleLogicalExpression, Box<LogicalExpression>)),
}

#[derive(Debug, Clone)]
pub(crate) enum SimpleLogicalExpression {
    Literal(Literal),
    Not(Box<LogicalExpression>),
    Rel(RelationalExpression),
}
#[derive(Debug, Clone)]
pub(crate) struct RelationalExpression {
    pub(crate) operator: RelationalOperator,
    pub(crate) operands: (RelationalTerm, RelationalTerm),
}

#[derive(Debug, Clone)]
pub(crate) enum RelationalOperator {
    Smaller,
    Larger,
    SmallerEq,
    LargerEq,
    Equal,
    NotEqual,
    Unify,
}

#[derive(Debug, Clone)]
pub(crate) enum RelationalTerm {
    Literal(Literal),
    Arithm(ArithmeticExpression),
}

#[derive(Debug, Clone)]
pub(crate) struct ArithmeticExpression {
    pub(crate) lhs: ArithmeticTerm,
    pub(crate) rhs: Option<(PlusMin, Box<ArithmeticExpression>)>,
}

#[derive(Debug, Clone)]
pub(crate) enum PlusMin {
    Plus,
    Min,
}

#[derive(Debug, Clone)]
pub(crate) struct ArithmeticTerm {
    pub(crate) lhs: ArithmeticFactor,
    pub(crate) rhs: Option<(DivMul, Box<ArithmeticTerm>)>,
}

#[derive(Debug, Clone)]
pub(crate) enum DivMul {
    Division,
    Multiplication,
}

#[derive(Debug, Clone)]
pub(crate) enum ArithmeticFactor {
    Number(f32),
    Variable(Variable),
    Neg(Box<ArithmeticFactor>),
    Group(Box<ArithmeticExpression>),
}

#[derive(Debug, Clone)]
pub(crate) struct Body(pub(crate) Box<[Spanned<BodyFormula>]>);

#[derive(Debug, Clone)]
pub(crate) enum BodyFormula {
    BeliefOrGoal {
        trigger: BodyFormulaTrigger,
        literal: Literal,
    },
    Action(Spanned<Action>),
}

#[derive(Debug, Clone)]
pub(crate) enum BodyFormulaTrigger {
    Add,
    Remove,
    Achieve,
    Query,
}

#[derive(Debug, Clone)]
pub(crate) enum Action {
    Builtin(BuiltinAction),
    User(Spanned<AtomicFormula>),
}
