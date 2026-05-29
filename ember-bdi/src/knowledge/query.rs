use alloc::boxed::Box;
use alloc::collections::btree_map::Iter;
use alloc::vec::Vec;

use crate::bindings::Bindings;
use crate::literal::Literal;
use crate::plan::RelationalQueryFormula;

use super::belief::{BeliefMetadata, NormalizedBelief};
use super::store::BeliefBase;

use self::formula::eval::EvaluationError;

/// Lazy resolution of a query formula.
#[derive(Debug, Clone)]
pub struct Query<'a> {
    conjunctions: Box<[Conjunction<'a>]>,
}

impl<'a> Query<'a> {
    pub fn next_bindings(
        &mut self,
        existing_bindings: Option<&Bindings<'a>>,
    ) -> Option<Bindings<'a>> {
        for conjunction in self.conjunctions.iter_mut() {
            let Some(bindings) = conjunction.next_bindings(existing_bindings) else {
                continue;
            };
            return Some(bindings);
        }
        None
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Conjunction<'a> {
    operands: Box<[GroundQuery<'a>]>,
}

impl<'a> Conjunction<'a> {
    fn next_bindings(&mut self, existing_bindings: Option<&Bindings<'a>>) -> Option<Bindings<'a>> {
        let mut current_bindings = Vec::with_capacity(self.operands.len());
        let mut cursor = 0_usize;
        while let Some(operand) = self.operands.get_mut(cursor) {
            match operand.next_bindings(
                current_bindings
                    .get(cursor.saturating_sub(1))
                    .or(existing_bindings),
            ) {
                Some(bindings) => {
                    current_bindings.push(bindings);
                    cursor += 1;
                }
                None => {
                    if cursor == 0 {
                        break;
                    }
                    current_bindings.pop();
                    operand.reset();
                    cursor -= 1;
                }
            }
        }

        // Reset every operand except the first one in case this function gets
        // called again.
        self.operands.iter_mut().skip(1).for_each(|o| o.reset());

        current_bindings.pop()
    }
}

#[derive(Debug, Clone)]
pub(crate) struct GroundQuery<'a> {
    /// Closed-world principle of "not". If the query is not satisfyable with
    /// any bindings, it succeeds.
    negated: bool,
    beliefs: Option<Iter<'a, NormalizedBelief, BeliefMetadata>>,
    operand: QueryOperand<'a>,

    /// On backtracking, the beliefs it has already tried have to be redone.
    original: Option<Iter<'a, NormalizedBelief, BeliefMetadata>>,
}

impl<'a> GroundQuery<'a> {
    fn next_bindings(&mut self, existing_bindings: Option<&Bindings<'a>>) -> Option<Bindings<'a>> {
        match (
            self.negated,
            self.operand
                .next_bindings(self.beliefs.as_mut(), existing_bindings),
        ) {
            (false, r) => r,
            (true, Some(_)) => None,
            (true, None) => Some(
                // Ensure that empty bindings are always returned such that the
                // query does not fail.
                existing_bindings.cloned().unwrap_or_else(Bindings::empty),
            ),
        }
    }

    fn reset(&mut self) {
        self.beliefs = self.original.clone();
    }
}

#[derive(Debug, Clone)]
pub(crate) enum QueryOperand<'a> {
    Literal(&'a Literal),
    Relational(&'a RelationalQueryFormula),
}

impl<'a> QueryOperand<'a> {
    fn next_bindings(
        &mut self,
        beliefs: Option<&mut Iter<'a, NormalizedBelief, BeliefMetadata>>,
        existing_bindings: Option<&Bindings<'a>>,
    ) -> Option<Bindings<'a>> {
        use QueryOperand::*;

        match self {
            Literal(literal) => beliefs.and_then(|b| {
                b.find_map(|(b, m)| b.unify_literal(m, literal, existing_bindings).ok())
            }),
            Relational(formula) => formula.verify_bindings(existing_bindings).ok().flatten(),
        }
    }
}

pub trait IntoQuery<'a>
where
    Self: 'a,
{
    fn into_query(self, knowledge: &'a BeliefBase) -> Query<'a>;
}

impl RelationalQueryFormula {
    /// Checks that the given bindings can be used to fully evaluate the formula, and
    /// that the formula evaluates to true.
    pub(super) fn verify_bindings<'a>(
        &'a self,
        bindings: Option<&Bindings<'a>>,
    ) -> Result<Option<Bindings<'a>>, EvaluationError> {
        match bindings {
            Some(bindings) => formula::eval::evaluate_relational(self, bindings),
            None => formula::eval::evaluate_relational(self, &Bindings::empty()),
        }
    }
}

pub(crate) mod formula {
    use alloc::boxed::Box;

    use crate::literal::Literal;
    use crate::term::{Atom, Term};
    use crate::{knowledge::store::BeliefBase, term::Structure};

    use super::{IntoQuery, Query};

    #[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
    pub enum QueryFormula {
        Not(Box<QueryFormula>),
        Logical {
            operator: LogicalOperator,
            operands: Box<[QueryFormula]>,
        },
        Literal(Literal),
        Relational(RelationalQueryFormula),
    }

    impl<'a> IntoQuery<'a> for &'a QueryFormula {
        fn into_query(self, knowledge: &'a BeliefBase) -> Query<'a> {
            self::into_dnf::convert(self, knowledge)
        }
    }

    impl QueryFormula {
        pub fn and<const N: usize>(operands: [QueryFormula; N]) -> Self {
            QueryFormula::Logical {
                operator: LogicalOperator::Conjunction,
                operands: Box::new(operands),
            }
        }

        pub fn or<const N: usize>(operands: [QueryFormula; N]) -> Self {
            QueryFormula::Logical {
                operator: LogicalOperator::Disjunction,
                operands: Box::new(operands),
            }
        }

        pub fn negate(self) -> Self {
            Self::Not(Box::new(self))
        }

        pub fn literal(
            negated: bool,
            functor: impl Into<Atom>,
            arguments: Option<impl Into<Box<[Term]>>>,
        ) -> Self {
            Literal::Atom {
                negated,
                structure: Structure {
                    functor: functor.into(),
                    arguments: arguments.map(Into::into),
                },
            }
            .into()
        }
    }

    impl From<Literal> for QueryFormula {
        fn from(literal: Literal) -> Self {
            Self::Literal(literal)
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    pub enum LogicalOperator {
        Conjunction,
        Disjunction,
    }

    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
    pub struct RelationalQueryFormula {
        pub operator: RelationalOperator,
        pub operands: Box<(ArithmeticExpression, ArithmeticExpression)>,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    pub enum RelationalOperator {
        Compare {
            operator: CompareOperator,
            equal: bool,
        },
        Unify,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    pub enum CompareOperator {
        LessThan,
        GreaterThan,
        EqualTo,
    }

    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
    pub enum ArithmeticExpression {
        Term(Term),
        Operation {
            operator: ArithmeticOperator,
            operands: Box<[ArithmeticExpression]>,
        },
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    pub enum ArithmeticOperator {
        Sum,
        Min,
        Div,
        Mul,
    }

    pub(crate) mod eval {
        use crate::bindings::Bindings;
        use crate::term::view::TermView;
        use crate::term::{NonGround, Term, TotalCmpF32};
        use crate::unification::traits::UnifyView;

        use super::{
            ArithmeticExpression, ArithmeticOperator, CompareOperator, RelationalOperator,
            RelationalQueryFormula,
        };

        #[derive(Debug, Clone, Copy)]
        pub enum EvaluationError {
            InsufficientlyBound,
            TypeMismatch,
            DivisionByZero,
        }

        impl core::fmt::Display for EvaluationError {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                write!(
                    f,
                    "evaluation error: {}",
                    match self {
                        Self::InsufficientlyBound => "formula is insufficiently bound",
                        Self::TypeMismatch => "type mismatch: expected a number for arithmetic",
                        Self::DivisionByZero => "division by zero",
                    }
                )
            }
        }

        impl core::error::Error for EvaluationError {}

        /// Evaluates a relational formula.
        /// Returns `Ok(Some(Bindings))` if the relation holds (or unification succeeds).
        /// Returns `Ok(None)` if the relation is logically false (or unification fails).
        /// Returns `Err` if arithmetic evaluation fails (e.g., type mismatch, unbound var).
        pub fn evaluate_relational<'a>(
            formula: &'a RelationalQueryFormula,
            bindings: &Bindings<'a>,
        ) -> Result<Option<Bindings<'a>>, EvaluationError> {
            match formula.operator {
                RelationalOperator::Compare { operator, equal } => {
                    let left = evaluate_arithmetic(&formula.operands.0, bindings)?;
                    let right = evaluate_arithmetic(&formula.operands.1, bindings)?;

                    let (l, r) = (TotalCmpF32::from(left), TotalCmpF32::from(right));

                    let is_satisfied = match operator {
                        CompareOperator::LessThan => l < r,
                        CompareOperator::GreaterThan => l > r,
                        CompareOperator::EqualTo => l == r,
                    };

                    if is_satisfied || (equal && l == r) {
                        Ok(Some(bindings.clone()))
                    } else {
                        Ok(None)
                    }
                }
                RelationalOperator::Unify => {
                    let left = resolve_for_unification(&formula.operands.0, bindings)?;
                    let right = resolve_for_unification(&formula.operands.1, bindings)?;

                    match left.unify(right, Some(bindings)) {
                        Ok(new_bindings) => Ok(Some(new_bindings)),
                        Err(_) => Ok(None),
                    }
                }
            }
        }

        /// Evaluates an arithmetic expression to a concrete f32.
        pub fn evaluate_arithmetic(
            expr: &ArithmeticExpression,
            bindings: &Bindings,
        ) -> Result<f32, EvaluationError> {
            match expr {
                ArithmeticExpression::Term(term) => resolve_to_f32(term, bindings),
                ArithmeticExpression::Operation { operator, operands } => {
                    if operands.is_empty() {
                        return Ok(0.0);
                    }

                    let mut values = operands.iter().map(|o| evaluate_arithmetic(o, bindings));
                    let first = values.next().unwrap()?;

                    match operator {
                        ArithmeticOperator::Sum => values.try_fold(first, |acc, x| Ok(acc + x?)),

                        ArithmeticOperator::Min => {
                            // Unary minus if only 1 operand, otherwise sequential subtraction.
                            if operands.len() == 1 {
                                Ok(-first)
                            } else {
                                values.try_fold(first, |acc, x| Ok(acc - x?))
                            }
                        }
                        ArithmeticOperator::Mul => values.try_fold(first, |acc, x| Ok(acc * x?)),
                        ArithmeticOperator::Div => values.try_fold(first, |acc, x| {
                            let d = x?;
                            if d.abs() <= f32::EPSILON {
                                Err(EvaluationError::DivisionByZero)
                            } else {
                                Ok(acc / d)
                            }
                        }),
                    }
                }
            }
        }

        /// Resolves an expression to a Term for unification.
        /// Math operations are aggressively evaluated to numbers; raw terms are passed through.
        fn resolve_for_unification<'a>(
            expr: &'a ArithmeticExpression,
            bindings: &Bindings<'a>,
        ) -> Result<TermView<'a>, EvaluationError> {
            match expr {
                ArithmeticExpression::Term(t) => Ok(TermView::Term(t)),
                ArithmeticExpression::Operation { .. } => {
                    let val = evaluate_arithmetic(expr, bindings)?;
                    Ok(TermView::Number(val.into()))
                }
            }
        }

        /// Recursively traces a term through the bindings to extract an f32.
        fn resolve_to_f32(term: &Term, bindings: &Bindings) -> Result<f32, EvaluationError> {
            match term {
                Term::Number(n) => Ok(**n),
                Term::Variable(NonGround(v)) => match bindings.get(v) {
                    Some(TermView::Term(t)) => resolve_to_f32(t, bindings),
                    Some(TermView::Variable(v)) => {
                        resolve_to_f32(&Term::Variable(NonGround((*v).clone())), bindings)
                    }
                    Some(TermView::Literal { .. }) => Err(EvaluationError::TypeMismatch),
                    Some(TermView::Number(n)) => Ok(**n),
                    None => Err(EvaluationError::InsufficientlyBound),
                },
                _ => Err(EvaluationError::TypeMismatch),
            }
        }
    }

    /// AI-generated module for converting the recursive query formula structure into the
    /// negation-normal form compatible with the [`Query`] type.
    mod into_dnf {
        use alloc::vec;
        use alloc::vec::Vec;

        use crate::knowledge::query::{Conjunction, GroundQuery, Query, QueryOperand};
        use crate::knowledge::store::BeliefBase;

        use crate::literal::Literal;

        use super::{LogicalOperator, QueryFormula};

        /// Intermediate structure to build Disjunctive Normal Form: (A & B) | (C & D)
        struct DnfBuilder<'a>(Vec<Vec<GroundQuery<'a>>>);

        impl<'a> DnfBuilder<'a> {
            /// Identity for OR: An empty list of conjunctions
            fn empty() -> Self {
                Self(Vec::new())
            }

            /// Identity for AND: A single empty conjunction
            fn unit() -> Self {
                Self(vec![Vec::new()])
            }

            /// Logic OR: Combine the sets of possible conjunctions
            fn or(mut self, other: Self) -> Self {
                self.0.extend(other.0);
                self
            }

            /// Logic AND: Create a Cartesian product of all conjunctions (Distribution)
            fn and(self, other: Self) -> Self {
                let mut result = Vec::new();
                for left_conj in &self.0 {
                    for right_conj in &other.0 {
                        let mut combined = left_conj.clone();
                        combined.extend(right_conj.clone());
                        result.push(combined);
                    }
                }
                DnfBuilder(result)
            }
        }

        pub fn convert<'a>(formula: &'a QueryFormula, bb: &'a BeliefBase) -> Query<'a> {
            let dnf = transform(formula, false, bb);

            let conjunctions = dnf
                .0
                .into_iter()
                .map(|ops| Conjunction {
                    operands: ops.into_boxed_slice(),
                })
                .collect::<Vec<_>>()
                .into_boxed_slice();

            Query { conjunctions }
        }

        fn transform<'a>(
            formula: &'a QueryFormula,
            negated: bool,
            bb: &'a BeliefBase,
        ) -> DnfBuilder<'a> {
            match formula {
                QueryFormula::Literal(lit) => {
                    let leaf = create_leaf(QueryOperand::Literal(lit), negated, bb);
                    DnfBuilder(vec![vec![leaf]])
                }
                QueryFormula::Relational(rel) => {
                    let leaf = create_leaf(QueryOperand::Relational(rel), negated, bb);
                    DnfBuilder(vec![vec![leaf]])
                }
                QueryFormula::Not(inner) => transform(inner, !negated, bb),
                QueryFormula::Logical { operator, operands } => {
                    match (operator, negated) {
                        // Standard Disjunction OR Negated Conjunction (via De Morgan)
                        (LogicalOperator::Disjunction, false)
                        | (LogicalOperator::Conjunction, true) => operands
                            .iter()
                            .map(|op| transform(op, negated, bb))
                            .fold(DnfBuilder::empty(), DnfBuilder::or),
                        // Standard Conjunction OR Negated Disjunction (via De Morgan)
                        (LogicalOperator::Conjunction, false)
                        | (LogicalOperator::Disjunction, true) => operands
                            .iter()
                            .map(|op| transform(op, negated, bb))
                            .fold(DnfBuilder::unit(), DnfBuilder::and),
                    }
                }
            }
        }

        fn create_leaf<'a>(
            operand: QueryOperand<'a>,
            negated: bool,
            bb: &'a BeliefBase,
        ) -> GroundQuery<'a> {
            let beliefs = match operand {
                QueryOperand::Literal(Literal::Atom { structure, .. }) => bb
                    .beliefs
                    .get(&structure.atom_and_arity())
                    .map(|b| b.0.iter()),
                _ => None,
            };

            GroundQuery {
                negated,
                beliefs: beliefs.clone(),
                original: beliefs,
                operand,
            }
        }

        #[cfg(test)]
        mod tests {
            use super::*;
            use crate::literal::Literal;
            use crate::plan::{
                ArithmeticExpression, CompareOperator, LogicalOperator, QueryFormula,
                RelationalOperator, RelationalQueryFormula,
            };
            use crate::term::{Atom, Structure, Term};
            use alloc::boxed::Box;
            use alloc::vec;

            // --- Helpers ---

            fn mock_literal(name: &str) -> Literal {
                Literal::Atom {
                    negated: false,
                    structure: Structure {
                        functor: Atom(name.into()),
                        arguments: None,
                    },
                }
            }

            fn mock_relational() -> RelationalQueryFormula {
                RelationalQueryFormula {
                    operator: RelationalOperator::Compare {
                        operator: CompareOperator::EqualTo,
                        equal: true,
                    },
                    operands: Box::new((
                        ArithmeticExpression::Term(Term::Number(0.0.into())),
                        ArithmeticExpression::Term(Term::Number(0.0.into())),
                    )),
                }
            }

            // --- Tests ---

            #[test]
            fn compile_single_literal() {
                let bb = BeliefBase::default();
                let formula = QueryFormula::Literal(mock_literal("p"));

                let query = convert(&formula, &bb);

                assert_eq!(query.conjunctions.len(), 1);
                assert_eq!(query.conjunctions[0].operands.len(), 1);
                assert!(matches!(
                    query.conjunctions[0].operands[0].operand,
                    QueryOperand::Literal(_)
                ));
                assert!(!query.conjunctions[0].operands[0].negated);
            }

            #[test]
            fn double_negation_elimination() {
                // !!p  =>  p
                let bb = BeliefBase::default();
                let formula = QueryFormula::Not(Box::new(QueryFormula::Not(Box::new(
                    QueryFormula::Literal(mock_literal("p")),
                ))));

                let query = convert(&formula, &bb);

                assert_eq!(query.conjunctions.len(), 1);
                assert!(
                    !query.conjunctions[0].operands[0].negated,
                    "Double Not should be positive"
                );
            }

            #[test]
            fn de_morgan_not_and() {
                // !(p & q)  =>  (!p | !q)
                let bb = BeliefBase::default();
                let formula = QueryFormula::Not(Box::new(QueryFormula::Logical {
                    operator: LogicalOperator::Conjunction,
                    operands: vec![
                        QueryFormula::Literal(mock_literal("p")),
                        QueryFormula::Literal(mock_literal("q")),
                    ]
                    .into_boxed_slice(),
                }));

                let query = convert(&formula, &bb);

                // Should result in two separate conjunctions (disjunction)
                assert_eq!(query.conjunctions.len(), 2);
                assert!(query.conjunctions[0].operands[0].negated);
                assert!(query.conjunctions[1].operands[0].negated);
            }

            #[test]
            fn de_morgan_not_or() {
                // !(p | q)  =>  (!p & !q)
                let bb = BeliefBase::default();
                let formula = QueryFormula::Not(Box::new(QueryFormula::Logical {
                    operator: LogicalOperator::Disjunction,
                    operands: vec![
                        QueryFormula::Literal(mock_literal("p")),
                        QueryFormula::Literal(mock_literal("q")),
                    ]
                    .into_boxed_slice(),
                }));

                let query = convert(&formula, &bb);

                // Should result in one conjunction with two negated operands
                assert_eq!(query.conjunctions.len(), 1);
                assert_eq!(query.conjunctions[0].operands.len(), 2);
                assert!(query.conjunctions[0].operands[0].negated);
                assert!(query.conjunctions[0].operands[1].negated);
            }

            #[test]
            fn distributive_law_complex() {
                // (A | B) & (C | D)  =>  (A&C) | (A&D) | (B&C) | (B&D)
                let bb = BeliefBase::default();

                let left = QueryFormula::Logical {
                    operator: LogicalOperator::Disjunction,
                    operands: vec![
                        QueryFormula::Literal(mock_literal("a")),
                        QueryFormula::Literal(mock_literal("b")),
                    ]
                    .into_boxed_slice(),
                };
                let right = QueryFormula::Logical {
                    operator: LogicalOperator::Disjunction,
                    operands: vec![
                        QueryFormula::Literal(mock_literal("c")),
                        QueryFormula::Literal(mock_literal("d")),
                    ]
                    .into_boxed_slice(),
                };
                let formula = QueryFormula::Logical {
                    operator: LogicalOperator::Conjunction,
                    operands: vec![left, right].into_boxed_slice(),
                };

                let query = convert(&formula, &bb);

                // Verification of Cartesian product
                assert_eq!(query.conjunctions.len(), 4, "Should have 4 paths (2x2)");
                for conj in query.conjunctions.iter() {
                    assert_eq!(conj.operands.len(), 2, "Each path needs both terms");
                }
            }

            #[test]
            fn nested_relational_negation() {
                // p & !(x == 0)
                let bb = BeliefBase::default();
                let formula = QueryFormula::Logical {
                    operator: LogicalOperator::Conjunction,
                    operands: vec![
                        QueryFormula::Literal(mock_literal("p")),
                        QueryFormula::Not(Box::new(QueryFormula::Relational(mock_relational()))),
                    ]
                    .into_boxed_slice(),
                };

                let query = convert(&formula, &bb);

                assert_eq!(query.conjunctions.len(), 1);
                let ops = &query.conjunctions[0].operands;
                assert_eq!(ops.len(), 2);

                // First operand is p (positive)
                assert!(!ops[0].negated);
                assert!(matches!(ops[0].operand, QueryOperand::Literal(_)));

                // Second operand is Relational (negated)
                assert!(ops[1].negated);
                assert!(matches!(ops[1].operand, QueryOperand::Relational(_)));
            }

            #[test]
            fn deep_tree_flattening() {
                // (A & B) | (C & (D | E)) => (A&B) | (C&D) | (C&E)
                let bb = BeliefBase::default();

                let d_or_e = QueryFormula::Logical {
                    operator: LogicalOperator::Disjunction,
                    operands: vec![
                        QueryFormula::Literal(mock_literal("d")),
                        QueryFormula::Literal(mock_literal("e")),
                    ]
                    .into_boxed_slice(),
                };

                let c_and_de = QueryFormula::Logical {
                    operator: LogicalOperator::Conjunction,
                    operands: vec![QueryFormula::Literal(mock_literal("c")), d_or_e]
                        .into_boxed_slice(),
                };

                let ab = QueryFormula::Logical {
                    operator: LogicalOperator::Conjunction,
                    operands: vec![
                        QueryFormula::Literal(mock_literal("a")),
                        QueryFormula::Literal(mock_literal("b")),
                    ]
                    .into_boxed_slice(),
                };

                let formula = QueryFormula::Logical {
                    operator: LogicalOperator::Disjunction,
                    operands: vec![ab, c_and_de].into_boxed_slice(),
                };

                let query = convert(&formula, &bb);

                // Expected: 3 conjunctions
                assert_eq!(query.conjunctions.len(), 3);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use alloc::boxed::Box;
    use alloc::vec;
    use alloc::vec::Vec;

    use crate::knowledge::belief::Belief;
    use crate::knowledge::store::BeliefBase;
    use crate::literal::Literal;
    use crate::plan::{
        ArithmeticExpression, ArithmeticOperator, CompareOperator, LogicalOperator, QueryFormula,
        RelationalOperator, RelationalQueryFormula,
    };
    use crate::term::view::TermView;
    use crate::term::{Atom, Structure, Term};

    use crate::testing::*;

    use super::IntoQuery;

    // --- Helpers ---

    fn literal(functor: &str, args: Vec<Term>) -> QueryFormula {
        QueryFormula::Literal(crate::testing::literal(functor, args))
    }

    fn belief(functor: &str, args: Vec<Term>) -> Belief {
        let lit = Literal::Atom {
            negated: false,
            structure: Structure {
                functor: Atom(functor.into()),
                arguments: if args.is_empty() {
                    None
                } else {
                    Some(args.into_boxed_slice())
                },
            },
        }
        .try_into_ground()
        .expect("belief can only contain ground literals");

        lit.into()
    }

    fn and(ops: Vec<QueryFormula>) -> QueryFormula {
        QueryFormula::Logical {
            operator: LogicalOperator::Conjunction,
            operands: ops.into_boxed_slice(),
        }
    }

    fn or(ops: Vec<QueryFormula>) -> QueryFormula {
        QueryFormula::Logical {
            operator: LogicalOperator::Disjunction,
            operands: ops.into_boxed_slice(),
        }
    }

    fn not(op: QueryFormula) -> QueryFormula {
        QueryFormula::Not(Box::new(op))
    }

    fn expr(t: Term) -> ArithmeticExpression {
        ArithmeticExpression::Term(t)
    }

    fn math(op: ArithmeticOperator, args: Vec<ArithmeticExpression>) -> ArithmeticExpression {
        ArithmeticExpression::Operation {
            operator: op,
            operands: args.into_boxed_slice(),
        }
    }

    fn cmp(
        l: ArithmeticExpression,
        op: CompareOperator,
        eq: bool,
        r: ArithmeticExpression,
    ) -> QueryFormula {
        QueryFormula::Relational(RelationalQueryFormula {
            operator: RelationalOperator::Compare {
                operator: op,
                equal: eq,
            },
            operands: Box::new((l, r)),
        })
    }

    fn unify(l: ArithmeticExpression, r: ArithmeticExpression) -> QueryFormula {
        QueryFormula::Relational(RelationalQueryFormula {
            operator: RelationalOperator::Unify,
            operands: Box::new((l, r)),
        })
    }

    // --- Tests ---

    #[test]
    fn shared_variable_conjunction() {
        let mut bb = BeliefBase::default();
        bb.assert(belief("parent", vec![s("alice"), s("bob")]));
        bb.assert(belief("parent", vec![s("bob"), s("charlie")]));

        let (x, y) = (v(), v());
        let formula = and(vec![
            literal("parent", vec![s("alice"), vt(&x)]),
            literal("parent", vec![vt(&x), vt(&y)]),
        ]);

        let mut query = (&formula).into_query(&bb);
        let bindings = query.next_bindings(None).expect("Should find bindings");

        assert_eq!(bindings.get(&x), Some(&s("bob").as_view()));
        assert_eq!(bindings.get(&y), Some(&s("charlie").as_view()));
    }

    #[test]
    fn backtracking_across_operands() {
        let mut bb = BeliefBase::default();
        bb.assert(belief("p", vec![n(1.0), n(10.0)]));
        bb.assert(belief("p", vec![n(1.0), n(20.0)]));
        bb.assert(belief("q", vec![n(20.0), n(30.0)]));

        let (x, y) = (v(), v());
        let formula = and(vec![
            literal("p", vec![n(1.0), vt(&x)]),
            literal("q", vec![vt(&x), vt(&y)]),
        ]);

        let mut query = (&formula).into_query(&bb);
        let bindings = query.next_bindings(None).expect("Should backtrack to X=20");

        assert_eq!(bindings.get(&x), Some(&n(20.0).as_view()));
        assert_eq!(bindings.get(&y), Some(&n(30.0).as_view()));
    }

    #[test]
    fn closed_world_negation() {
        let mut bb = BeliefBase::default();
        bb.assert(belief("is_raining", vec![]));

        let f_sunny = not(literal("is_sunny", vec![]));
        assert!((&f_sunny).into_query(&bb).next_bindings(None).is_some());

        let f_raining = not(literal("is_raining", vec![]));
        assert!((&f_raining).into_query(&bb).next_bindings(None).is_none());
    }

    #[test]
    fn disjunction_and_flattening() {
        let mut bb = BeliefBase::default();
        bb.assert(belief("a", vec![n(1.0)]));
        bb.assert(belief("b", vec![n(2.0)]));
        bb.assert(belief("c", vec![n(2.0)]));

        let x = v();
        // (a(X) | b(X)) & c(X) -> Should bind X=2
        let formula = and(vec![
            or(vec![literal("a", vec![vt(&x)]), literal("b", vec![vt(&x)])]),
            literal("c", vec![vt(&x)]),
        ]);

        let mut query = (&formula).into_query(&bb);
        let bindings = query.next_bindings(None).expect("Should match X=2");
        assert_eq!(bindings.get(&x), Some(&n(2.0).as_view()));
        assert!(query.next_bindings(None).is_none());
    }

    #[test]
    fn relational_comparison() {
        let mut bb = BeliefBase::default();
        bb.assert(belief("val", vec![n(5.0)]));
        bb.assert(belief("val", vec![n(15.0)]));

        let x = v();
        // val(X) & X > 10
        let formula = and(vec![
            literal("val", vec![vt(&x)]),
            cmp(
                expr(vt(&x)),
                CompareOperator::GreaterThan,
                false,
                expr(n(10.0)),
            ),
        ]);

        let mut query = (&formula).into_query(&bb);
        let bindings = query.next_bindings(None).expect("Should find X=15");
        assert_eq!(bindings.get(&x), Some(&n(15.0).as_view()));
        assert!(query.next_bindings(None).is_none());
    }

    #[test]
    fn relational_unification_math() {
        let mut bb = BeliefBase::default();
        bb.assert(belief("base", vec![n(10.0)]));

        let (x, y) = (v(), v());
        // base(X) & Y = X * 2
        let formula = and(vec![
            literal("base", vec![vt(&x)]),
            unify(
                expr(vt(&y)),
                math(ArithmeticOperator::Mul, vec![expr(vt(&x)), expr(n(2.0))]),
            ),
        ]);

        let mut query = (&formula).into_query(&bb);
        let bindings = query.next_bindings(None).expect("Should unify Y to 20");
        assert_eq!(bindings.get(&y), Some(&TermView::Number(20.0.into())));
    }

    #[test]
    fn arithmetic_division_by_zero_fails_gracefully() {
        let mut bb = BeliefBase::default();
        bb.assert(belief("val", vec![n(0.0)]));
        bb.assert(belief("val", vec![n(2.0)]));

        let x = v();
        // val(X) & (10 / X) == 5
        let formula = and(vec![
            literal("val", vec![vt(&x)]),
            cmp(
                math(ArithmeticOperator::Div, vec![expr(n(10.0)), expr(vt(&x))]),
                CompareOperator::EqualTo,
                true,
                expr(n(5.0)),
            ),
        ]);

        let mut query = (&formula).into_query(&bb);
        // The Div by 0 branch should return None natively and backtrack to X=2
        let bindings = query
            .next_bindings(None)
            .expect("Should recover and find X=2");
        assert_eq!(bindings.get(&x), Some(&n(2.0).as_view()));
        assert!(query.next_bindings(None).is_none());
    }

    #[test]
    fn type_mismatch_fails_gracefully() {
        let mut bb = BeliefBase::default();
        bb.assert(belief("val", vec![s("not_a_number")]));

        let x = v();
        // val(X) & X > 0
        let formula = and(vec![
            literal("val", vec![vt(&x)]),
            cmp(
                expr(vt(&x)),
                CompareOperator::GreaterThan,
                false,
                expr(n(0.0)),
            ),
        ]);

        let mut query = (&formula).into_query(&bb);
        assert!(query.next_bindings(None).is_none());
    }

    #[test]
    fn complex_de_morgan_resolution() {
        let mut bb = BeliefBase::default();
        bb.assert(belief("p", vec![n(1.0)]));
        bb.assert(belief("q", vec![n(1.0)]));

        // !( !p(1) | !q(1) ) => p(1) & q(1)
        let formula = not(or(vec![
            not(literal("p", vec![n(1.0)])),
            not(literal("q", vec![n(1.0)])),
        ]));

        let mut query = (&formula).into_query(&bb);
        assert!(query.next_bindings(None).is_some());
    }
}
