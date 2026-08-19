use alloc::boxed::Box;
use alloc::vec::Vec;

use ember_collections::SmallSetIter as Iter;

use crate::bindings::Bindings;
use crate::literal::Literal;
use crate::plan::RelationalQueryFormula;
use crate::unification::error::UnificationError;

use super::base::KnowledgeBase;
use super::belief::Knowledge;

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

    fn reset(&mut self) {
        self.conjunctions.iter_mut().for_each(Conjunction::reset);
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Conjunction<'a> {
    operands: Box<[GroundQuery<'a>]>,
    /// Bindings for operands already satisfied in the current search.
    /// Persists across calls so a later call resumes the walk instead of
    /// restarting it at operand 0.
    current_bindings: Vec<Bindings<'a>>,
}

impl<'a> Conjunction<'a> {
    fn next_bindings(&mut self, existing_bindings: Option<&Bindings<'a>>) -> Option<Bindings<'a>> {
        // Resume: back off the last operand so its next alternative gets
        // tried, instead of restarting the whole walk at operand 0.
        let mut cursor = self.current_bindings.len();
        if cursor > 0 && cursor == self.operands.len() {
            cursor -= 1;
            self.current_bindings.pop();
        }

        while let Some(operand) = self.operands.get_mut(cursor) {
            match operand.next_bindings(
                self.current_bindings
                    .get(cursor.saturating_sub(1))
                    .or(existing_bindings),
            ) {
                Some(bindings) => {
                    self.current_bindings.push(bindings);
                    cursor += 1;
                }
                None => {
                    if cursor == 0 {
                        return None;
                    }
                    self.current_bindings.pop();
                    operand.reset();
                    cursor -= 1;
                }
            }
        }

        self.current_bindings.last().cloned()
    }

    /// Full reset for reuse under different outer bindings, as opposed to
    /// `next_bindings`'s resume-in-place backtracking.
    fn reset(&mut self) {
        self.current_bindings.clear();
        self.operands.iter_mut().for_each(GroundQuery::reset);
    }
}

#[derive(Debug, Clone)]
pub(crate) struct GroundQuery<'a> {
    /// Closed-world principle of "not". If the query is not satisfyable with
    /// any bindings, it succeeds.
    negated: bool,
    beliefs: Option<Iter<'a, Knowledge>>,
    operand: QueryOperand<'a>,

    /// On backtracking, the beliefs it has already tried have to be redone.
    original: Option<Iter<'a, Knowledge>>,

    /// To resolve rules, the beliefbase has to be queried recursively.
    knowledge: &'a KnowledgeBase,

    /// During backtracking a ground query needs to know whether it has already been
    /// evaluated before in cases where it can produce bindings infinitely. For example, where
    /// there are not beliefs to go through or regular operators such as compare or unify,
    /// regardless of the outcome. If it has and the backtracking engine comes back with "do you
    /// have any other ways to satisfy yourself?" it should return `None`.
    evaluated: bool,
}

impl<'a> GroundQuery<'a> {
    fn next_bindings(&mut self, existing_bindings: Option<&Bindings<'a>>) -> Option<Bindings<'a>> {
        if self.evaluated {
            return None;
        }

        match (
            self.negated,
            self.operand
                .next_bindings(self.beliefs.as_mut(), existing_bindings, self.knowledge),
        ) {
            (false, result) => result,
            (true, bindings) => {
                // Don't let a later ask (without an intervening `reset`) re-run
                // the operand, whose own belief iterators may have been
                // exhausted by this very evaluation and would then wrongly
                // report a fresh success.
                self.evaluated = true;

                if bindings.is_some() {
                    None
                } else {
                    Some(
                        // Ensure that empty bindings are always returned such that the
                        // query does not fail.
                        existing_bindings.cloned().unwrap_or_else(Bindings::empty),
                    )
                }
            }
        }
    }

    fn reset(&mut self) {
        self.beliefs = self.original.clone();
        self.evaluated = false;
        self.operand.reset();
    }
}

#[derive(Debug, Clone)]
pub(crate) enum QueryOperand<'a> {
    Literal {
        literal: &'a Literal,
        /// During unification of this literal it might be that we need to query the
        /// knowledge base again to prove a belief rule. This query has to be
        /// back-trackable, hence we store it here.
        rule_in_process: Option<RuleQuery<'a>>,
    },
    Relational {
        formula: &'a RelationalQueryFormula,
        /// Because this query is not based on a belief iterator, it can return `Some` infinitely as
        /// if it has new solutions. Using `evaluated`, `None` can be returned after the first call.
        evaluated: bool,
    },
    Group(Box<Query<'a>>),
}

impl<'a> QueryOperand<'a> {
    fn literal(literal: &'a Literal) -> Self {
        Self::Literal {
            literal,
            rule_in_process: None,
        }
    }

    fn reset(&mut self) {
        match self {
            Self::Literal {
                rule_in_process, ..
            } => *rule_in_process = None,
            Self::Group(query) => query.reset(),
            Self::Relational { evaluated, .. } => *evaluated = false,
        }
    }

    fn next_bindings(
        &mut self,
        beliefs: Option<&mut Iter<'a, Knowledge>>,
        existing_bindings: Option<&Bindings<'a>>,
        knowledge_base: &'a KnowledgeBase,
    ) -> Option<Bindings<'a>> {
        use crate::unification::traits::Unify;

        match self {
            QueryOperand::Literal {
                literal,
                rule_in_process,
            } => rule_in_process
                .as_mut()
                .and_then(|rule_query| rule_query.next_bindings(literal, existing_bindings))
                .or_else(|| {
                    beliefs.and_then(|b| {
                        b.find_map(|knowledge| {
                            if let Some(rule) = &knowledge.rule {
                                let body = knowledge_base.query(rule);
                                let mut rule_query = RuleQuery::new(
                                    &knowledge.belief,
                                    literal,
                                    body,
                                    existing_bindings,
                                )
                                .ok()?;
                                let result = rule_query.next_bindings(literal, existing_bindings);
                                *rule_in_process = Some(rule_query);
                                result
                            } else {
                                knowledge.belief.unify(literal, existing_bindings).ok()
                            }
                        })
                    })
                }),
            QueryOperand::Relational { formula, evaluated } => {
                if *evaluated {
                    None
                } else {
                    *evaluated = true;
                    formula.verify_bindings(existing_bindings).ok().flatten()
                }
            }
            QueryOperand::Group(query) => query.next_bindings(existing_bindings),
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct RuleQuery<'a> {
    /// The rule's head literal.
    head: &'a Literal,
    /// The rule's body, queried lazily over the knowledge base.
    body: Query<'a>,
    /// Bindings resulting from unification of the rules head with the parent queries literal.
    head_bindings: Bindings<'a>,
}

impl<'a> RuleQuery<'a> {
    fn new(
        head: &'a Literal,
        literal: &'a Literal,
        body: Query<'a>,
        existing_bindings: Option<&Bindings<'a>>,
    ) -> Result<Self, UnificationError> {
        use crate::unification::traits::Unify;

        let mut head_bindings = head.unify(literal, existing_bindings)?;
        head_bindings.retain_variables(&head.variables());

        Ok(Self {
            head,
            body,
            head_bindings,
        })
    }

    fn next_bindings(
        &mut self,
        literal: &'a Literal,
        existing_bindings: Option<&Bindings<'a>>,
    ) -> Option<Bindings<'a>> {
        use crate::unification::traits::Unify;

        while let Some(mut bindings) = self.body.next_bindings(Some(&self.head_bindings)) {
            // Drop all variables that are not mentioned in the head of the rule.
            bindings.retain_variables(&self.head.variables());

            // Restablish the connection (aliasing) between variables in the rule and the original
            // literal.
            let Ok(bindings) = self.head.unify(literal, Some(&bindings)) else {
                continue;
            };

            // Merge the surrounding bindings into the current ones.
            let bindings = match existing_bindings {
                Some(surrounding) => {
                    let Ok(bindings) = Bindings::merge_views([&bindings, surrounding]) else {
                        continue;
                    };
                    bindings
                }
                None => bindings,
            };

            return Some(bindings);
        }

        None
    }
}

pub trait IntoQuery<'a>
where
    Self: 'a,
{
    fn into_query(self, knowledge: &'a KnowledgeBase) -> Query<'a>;
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

    use crate::knowledge::base::KnowledgeBase;
    use crate::literal::Literal;
    use crate::term::{Atom, Structure, Term};

    use super::{IntoQuery, Query};

    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
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
        fn into_query(self, knowledge: &'a KnowledgeBase) -> Query<'a> {
            self::lowering::convert(self, knowledge)
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
            Literal {
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
        pub operands: (ArithmeticExpression, ArithmeticExpression),
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
        use ember_util::cmp::TotalCmpF32;

        use crate::bindings::Bindings;
        use crate::term::Term;
        use crate::term::view::TermView;
        use crate::unification::traits::UnifyView;

        use super::{
            ArithmeticExpression, ArithmeticOperator, CompareOperator, RelationalOperator,
            RelationalQueryFormula,
        };

        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
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
                Term::Variable(v) => match bindings.get_view(v) {
                    Some(TermView::Term(t)) => resolve_to_f32(t, bindings),
                    Some(TermView::Variable(v)) => {
                        resolve_to_f32(&Term::Variable((*v).clone()), bindings)
                    }
                    Some(TermView::Literal { .. }) => Err(EvaluationError::TypeMismatch),
                    Some(TermView::List(_)) => Err(EvaluationError::TypeMismatch),
                    Some(TermView::Number(n)) => Ok(**n),
                    Some(TermView::String(_)) => Err(EvaluationError::TypeMismatch),
                    None => Err(EvaluationError::InsufficientlyBound),
                },
                _ => Err(EvaluationError::TypeMismatch),
            }
        }
    }

    /// AI-generated
    /// Lowers a [`QueryFormula`] onto the [`Query`]/[`Conjunction`] shape
    /// directly, without distributing into disjunctive normal form. A
    /// formula that does not fit the shape expected at a given level (a
    /// compound formula in leaf position, negated or not) becomes its own
    /// subquery via [`QueryOperand::Group`] instead. Per-literal De Morgan
    /// distribution is unsound for a negated conjunction whose conjuncts
    /// share a variable, so it is not applied anywhere here, not even for
    /// the negated-disjunction case where it would be sound.
    mod lowering {
        use alloc::boxed::Box;
        use alloc::vec;
        use alloc::vec::Vec;

        use crate::knowledge::base::KnowledgeBase;
        use crate::knowledge::query::{Conjunction, GroundQuery, Query, QueryOperand};

        use crate::literal::Literal;

        use super::{LogicalOperator, QueryFormula};

        pub fn convert<'a>(formula: &'a QueryFormula, bb: &'a KnowledgeBase) -> Query<'a> {
            transform(formula, bb)
        }

        fn transform<'a>(formula: &'a QueryFormula, bb: &'a KnowledgeBase) -> Query<'a> {
            match formula {
                QueryFormula::Logical {
                    operator: LogicalOperator::Disjunction,
                    operands,
                } => Query {
                    conjunctions: operands
                        .iter()
                        .map(|op| transform_conjunction(op, bb))
                        .collect::<Vec<_>>()
                        .into_boxed_slice(),
                },
                // Does not start with an OR: it is a single AND-branch on its own.
                _ => Query {
                    conjunctions: vec![transform_conjunction(formula, bb)].into_boxed_slice(),
                },
            }
        }

        fn transform_conjunction<'a>(
            formula: &'a QueryFormula,
            bb: &'a KnowledgeBase,
        ) -> Conjunction<'a> {
            match formula {
                QueryFormula::Logical {
                    operator: LogicalOperator::Conjunction,
                    operands,
                } => Conjunction {
                    operands: operands
                        .iter()
                        .map(|op| transform_leaf(op, bb))
                        .collect::<Vec<_>>()
                        .into_boxed_slice(),
                    current_bindings: Vec::new(),
                },
                // Does not start with an AND: it is a single leaf on its own.
                _ => Conjunction {
                    operands: vec![transform_leaf(formula, bb)].into_boxed_slice(),
                    current_bindings: Vec::new(),
                },
            }
        }

        fn transform_leaf<'a>(formula: &'a QueryFormula, bb: &'a KnowledgeBase) -> GroundQuery<'a> {
            match formula {
                QueryFormula::Literal(lit) => create_leaf(QueryOperand::literal(lit), false, bb),
                QueryFormula::Relational(rel) => create_leaf(
                    QueryOperand::Relational {
                        formula: rel,
                        evaluated: false,
                    },
                    false,
                    bb,
                ),
                QueryFormula::Not(inner) => match inner.as_ref() {
                    // A negated literal/relational is still a plain ground leaf.
                    QueryFormula::Literal(lit) => create_leaf(QueryOperand::literal(lit), true, bb),
                    QueryFormula::Relational(rel) => create_leaf(
                        QueryOperand::Relational {
                            formula: rel,
                            evaluated: false,
                        },
                        true,
                        bb,
                    ),
                    // Negating a compound formula does not fit a leaf: resolve it
                    // as its own subquery and negate the existence check as a
                    // whole, rather than distributing the negation into it.
                    compound => create_group_leaf(compound, true, bb),
                },
                // A bare compound formula in leaf position, for example the
                // `B|C` inside `A & (B|C)`, does not fit a leaf either.
                compound @ QueryFormula::Logical { .. } => create_group_leaf(compound, false, bb),
            }
        }

        fn create_group_leaf<'a>(
            formula: &'a QueryFormula,
            negated: bool,
            bb: &'a KnowledgeBase,
        ) -> GroundQuery<'a> {
            GroundQuery {
                negated,
                beliefs: None,
                original: None,
                operand: QueryOperand::Group(Box::new(transform(formula, bb))),
                knowledge: bb,
                evaluated: false,
            }
        }

        fn create_leaf<'a>(
            operand: QueryOperand<'a>,
            negated: bool,
            bb: &'a KnowledgeBase,
        ) -> GroundQuery<'a> {
            let beliefs = match operand {
                QueryOperand::Literal {
                    literal: Literal { structure, .. },
                    ..
                } => bb
                    .collections
                    .get(&structure.atom_and_arity())
                    .map(|b| b.0.iter()),
                _ => None,
            };

            GroundQuery {
                negated,
                beliefs: beliefs.clone(),
                original: beliefs,
                operand,
                knowledge: bb,
                evaluated: false,
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
                Literal {
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
                    operands: (
                        ArithmeticExpression::Term(Term::Number(0.0.into())),
                        ArithmeticExpression::Term(Term::Number(0.0.into())),
                    ),
                }
            }

            // --- Tests ---

            #[test]
            fn single_literal_is_one_conjunction_one_leaf() {
                let bb = KnowledgeBase::default();
                let formula = QueryFormula::Literal(mock_literal("p"));

                let query = convert(&formula, &bb);

                assert_eq!(query.conjunctions.len(), 1);
                assert_eq!(query.conjunctions[0].operands.len(), 1);
                assert!(matches!(
                    query.conjunctions[0].operands[0].operand,
                    QueryOperand::Literal { .. }
                ));
                assert!(!query.conjunctions[0].operands[0].negated);
            }

            #[test]
            fn negated_literal_stays_a_plain_leaf() {
                // `not p` fits a leaf directly: no subquery needed.
                let bb = KnowledgeBase::default();
                let formula = QueryFormula::Not(Box::new(QueryFormula::Literal(mock_literal("p"))));

                let query = convert(&formula, &bb);

                assert_eq!(query.conjunctions.len(), 1);
                assert_eq!(query.conjunctions[0].operands.len(), 1);
                assert!(matches!(
                    query.conjunctions[0].operands[0].operand,
                    QueryOperand::Literal { .. }
                ));
                assert!(query.conjunctions[0].operands[0].negated);
            }

            #[test]
            fn negated_conjunction_becomes_a_single_subquery_leaf() {
                // `not (p & q)` no longer decomposes via De Morgan: it maps
                // onto one negated Group leaf wrapping the positive `p & q`
                // as its own nested query.
                let bb = KnowledgeBase::default();
                let formula = QueryFormula::Not(Box::new(QueryFormula::Logical {
                    operator: LogicalOperator::Conjunction,
                    operands: vec![
                        QueryFormula::Literal(mock_literal("p")),
                        QueryFormula::Literal(mock_literal("q")),
                    ]
                    .into_boxed_slice(),
                }));

                let query = convert(&formula, &bb);

                assert_eq!(query.conjunctions.len(), 1);
                assert_eq!(query.conjunctions[0].operands.len(), 1);
                let leaf = &query.conjunctions[0].operands[0];
                assert!(leaf.negated);
                let QueryOperand::Group(inner) = &leaf.operand else {
                    panic!("expected a Group leaf");
                };
                assert_eq!(inner.conjunctions.len(), 1);
                assert_eq!(inner.conjunctions[0].operands.len(), 2);
            }

            #[test]
            fn negated_disjunction_also_becomes_a_subquery_leaf() {
                // `not (p | q)` is sound to flatten via De Morgan, but the
                // lowering applies the same "compound under not becomes a
                // subquery" rule uniformly, without that exception.
                let bb = KnowledgeBase::default();
                let formula = QueryFormula::Not(Box::new(QueryFormula::Logical {
                    operator: LogicalOperator::Disjunction,
                    operands: vec![
                        QueryFormula::Literal(mock_literal("p")),
                        QueryFormula::Literal(mock_literal("q")),
                    ]
                    .into_boxed_slice(),
                }));

                let query = convert(&formula, &bb);

                assert_eq!(query.conjunctions.len(), 1);
                assert_eq!(query.conjunctions[0].operands.len(), 1);
                let leaf = &query.conjunctions[0].operands[0];
                assert!(leaf.negated);
                let QueryOperand::Group(inner) = &leaf.operand else {
                    panic!("expected a Group leaf");
                };
                assert_eq!(inner.conjunctions.len(), 2);
            }

            #[test]
            fn disjunction_nested_in_conjunction_does_not_distribute() {
                // `(a|b) & (c|d)` used to expand into a 4-way cartesian
                // product. It now maps onto one conjunction whose two
                // operands are each a Group leaf wrapping its own
                // disjunction, with no distribution at all.
                let bb = KnowledgeBase::default();

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

                assert_eq!(query.conjunctions.len(), 1);
                assert_eq!(query.conjunctions[0].operands.len(), 2);
                for leaf in query.conjunctions[0].operands.iter() {
                    assert!(!leaf.negated);
                    assert!(matches!(leaf.operand, QueryOperand::Group(_)));
                }
            }

            #[test]
            fn nested_relational_negation() {
                // p & !(x == 0)
                let bb = KnowledgeBase::default();
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
                assert!(matches!(ops[0].operand, QueryOperand::Literal { .. }));

                // Second operand is Relational (negated)
                assert!(ops[1].negated);
                assert!(matches!(ops[1].operand, QueryOperand::Relational { .. }));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use alloc::boxed::Box;
    use alloc::vec;
    use alloc::vec::Vec;

    use crate::knowledge::base::KnowledgeBase;
    use crate::knowledge::belief::Knowledge;
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

    fn belief(functor: &str, args: Vec<Term>) -> Knowledge {
        let lit = Literal {
            negated: false,
            structure: Structure {
                functor: Atom(functor.into()),
                arguments: if args.is_empty() {
                    None
                } else {
                    Some(args.into_boxed_slice())
                },
            },
        };

        lit.into()
    }

    fn rule(functor: &str, args: Vec<Term>, body: QueryFormula) -> Knowledge {
        let lit = Literal {
            negated: false,
            structure: Structure {
                functor: Atom(functor.into()),
                arguments: if args.is_empty() {
                    None
                } else {
                    Some(args.into_boxed_slice())
                },
            },
        };

        (lit, body).into()
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
            operands: (l, r),
        })
    }

    fn unify(l: ArithmeticExpression, r: ArithmeticExpression) -> QueryFormula {
        QueryFormula::Relational(RelationalQueryFormula {
            operator: RelationalOperator::Unify,
            operands: (l, r),
        })
    }

    // --- Tests ---

    #[test]
    fn shared_variable_conjunction() {
        let mut bb = KnowledgeBase::default();
        bb.assert_no_event(belief("parent", vec![string("alice"), string("bob")]));
        bb.assert_no_event(belief("parent", vec![string("bob"), string("charlie")]));

        let (x, y) = (variable(), variable());
        let formula = and(vec![
            literal("parent", vec![string("alice"), variable_term(&x)]),
            literal("parent", vec![variable_term(&x), variable_term(&y)]),
        ]);

        let mut query = (&formula).into_query(&bb);
        let bindings = query.next_bindings(None).expect("Should find bindings");

        assert_eq!(bindings.get_view(&x), Some(&string("bob").as_view()));
        assert_eq!(bindings.get_view(&y), Some(&string("charlie").as_view()));
    }

    #[test]
    fn backtracking_across_operands() {
        let mut bb = KnowledgeBase::default();
        bb.assert_no_event(belief("p", vec![number(1.0), number(10.0)]));
        bb.assert_no_event(belief("p", vec![number(1.0), number(20.0)]));
        bb.assert_no_event(belief("q", vec![number(20.0), number(30.0)]));

        let (x, y) = (variable(), variable());
        let formula = and(vec![
            literal("p", vec![number(1.0), variable_term(&x)]),
            literal("q", vec![variable_term(&x), variable_term(&y)]),
        ]);

        let mut query = (&formula).into_query(&bb);
        let bindings = query.next_bindings(None).expect("Should backtrack to X=20");

        assert_eq!(bindings.get_view(&x), Some(&number(20.0).as_view()));
        assert_eq!(bindings.get_view(&y), Some(&number(30.0).as_view()));
    }

    #[test]
    fn closed_world_negation() {
        let mut bb = KnowledgeBase::default();
        bb.assert_no_event(belief("is_raining", vec![]));

        let f_sunny = not(literal("is_sunny", vec![]));
        assert!((&f_sunny).into_query(&bb).next_bindings(None).is_some());

        let f_raining = not(literal("is_raining", vec![]));
        assert!((&f_raining).into_query(&bb).next_bindings(None).is_none());
    }

    #[test]
    fn disjunction_and_flattening() {
        let mut bb = KnowledgeBase::default();
        bb.assert_no_event(belief("a", vec![number(1.0)]));
        bb.assert_no_event(belief("b", vec![number(2.0)]));
        bb.assert_no_event(belief("c", vec![number(2.0)]));

        let x = variable();
        // (a(X) | b(X)) & c(X) -> Should bind X=2
        let formula = and(vec![
            or(vec![
                literal("a", vec![variable_term(&x)]),
                literal("b", vec![variable_term(&x)]),
            ]),
            literal("c", vec![variable_term(&x)]),
        ]);

        let mut query = (&formula).into_query(&bb);
        let bindings = query.next_bindings(None).expect("Should match X=2");
        assert_eq!(bindings.get_view(&x), Some(&number(2.0).as_view()));
        assert!(query.next_bindings(None).is_none());
    }

    #[test]
    fn relational_comparison() {
        let mut bb = KnowledgeBase::default();
        bb.assert_no_event(belief("val", vec![number(5.0)]));
        bb.assert_no_event(belief("val", vec![number(15.0)]));

        let x = variable();
        // val(X) & X > 10
        let formula = and(vec![
            literal("val", vec![variable_term(&x)]),
            cmp(
                expr(variable_term(&x)),
                CompareOperator::GreaterThan,
                false,
                expr(number(10.0)),
            ),
        ]);

        let mut query = (&formula).into_query(&bb);
        let bindings = query.next_bindings(None).expect("Should find X=15");
        assert_eq!(bindings.get_view(&x), Some(&number(15.0).as_view()));
        assert!(query.next_bindings(None).is_none());
    }

    #[test]
    fn relational_unification_math() {
        let mut bb = KnowledgeBase::default();
        bb.assert_no_event(belief("base", vec![number(10.0)]));

        let (x, y) = (variable(), variable());
        // base(X) & Y = X * 2
        let formula = and(vec![
            literal("base", vec![variable_term(&x)]),
            unify(
                expr(variable_term(&y)),
                math(
                    ArithmeticOperator::Mul,
                    vec![expr(variable_term(&x)), expr(number(2.0))],
                ),
            ),
        ]);

        let mut query = (&formula).into_query(&bb);
        let bindings = query.next_bindings(None).expect("Should unify Y to 20");
        assert_eq!(bindings.get_view(&y), Some(&TermView::Number(20.0.into())));
    }

    #[test]
    fn arithmetic_division_by_zero_fails_gracefully() {
        let mut bb = KnowledgeBase::default();
        bb.assert_no_event(belief("val", vec![number(0.0)]));
        bb.assert_no_event(belief("val", vec![number(2.0)]));

        let x = variable();
        // val(X) & (10 / X) == 5
        let formula = and(vec![
            literal("val", vec![variable_term(&x)]),
            cmp(
                math(
                    ArithmeticOperator::Div,
                    vec![expr(number(10.0)), expr(variable_term(&x))],
                ),
                CompareOperator::EqualTo,
                true,
                expr(number(5.0)),
            ),
        ]);

        let mut query = (&formula).into_query(&bb);
        // The Div by 0 branch should return None natively and backtrack to X=2
        let bindings = query
            .next_bindings(None)
            .expect("Should recover and find X=2");
        assert_eq!(bindings.get_view(&x), Some(&number(2.0).as_view()));
        assert!(query.next_bindings(None).is_none());
    }

    #[test]
    fn type_mismatch_fails_gracefully() {
        let mut bb = KnowledgeBase::default();
        bb.assert_no_event(belief("val", vec![string("not_a_number")]));

        let x = variable();
        // val(X) & X > 0
        let formula = and(vec![
            literal("val", vec![variable_term(&x)]),
            cmp(
                expr(variable_term(&x)),
                CompareOperator::GreaterThan,
                false,
                expr(number(0.0)),
            ),
        ]);

        let mut query = (&formula).into_query(&bb);
        assert!(query.next_bindings(None).is_none());
    }

    #[test]
    fn arithmetic_rejects_list_term() {
        let mut bb = KnowledgeBase::default();
        bb.assert_no_event(belief("val", vec![list(vec![number(1.0), number(2.0)])]));

        let x = variable();
        // val(X) & X > 0
        let formula = and(vec![
            literal("val", vec![variable_term(&x)]),
            cmp(
                expr(variable_term(&x)),
                CompareOperator::GreaterThan,
                false,
                expr(number(0.0)),
            ),
        ]);

        let mut query = (&formula).into_query(&bb);
        assert!(query.next_bindings(None).is_none());
    }

    #[test]
    fn complex_de_morgan_resolution() {
        let mut bb = KnowledgeBase::default();
        bb.assert_no_event(belief("p", vec![number(1.0)]));
        bb.assert_no_event(belief("q", vec![number(1.0)]));

        // !( !p(1) | !q(1) ) => p(1) & q(1)
        let formula = not(or(vec![
            not(literal("p", vec![number(1.0)])),
            not(literal("q", vec![number(1.0)])),
        ]));

        let mut query = (&formula).into_query(&bb);
        assert!(query.next_bindings(None).is_some());
    }

    // Currently fails: a rule's nested query is cached across calls
    // (`QueryOperand::Literal`'s `belief_to_process`), and `GroundQuery::reset`
    // only resets the belief iterator, not that cache. That's normally
    // harmless, because a leaf whose cached attempt genuinely fails falls
    // back to rebuilding its rule fresh (`.or_else` in
    // `QueryOperand::next_bindings`) -- which finds the still-true belief
    // again and "heals" the staleness. But when the stale, exhausted result
    // is wrapped in `not`, the negation turns "found nothing" into a
    // successful match instead of a failure, so the fallback that would
    // have rebuilt it fresh never runs, and the wrong answer survives.
    //
    // `alt(X) & green_safe` below: `alt` has two solutions. Trying X=1 first,
    // `green_safe` correctly fails (has_conflict is genuinely true). That
    // failure backtracks into `alt` for its second solution, X=2, and
    // retries `green_safe` -- whose cached search for `has_conflict` is
    // already spent on its one real match, finds nothing this time, and
    // `not` reports that as success, so the conjunction as a whole wrongly
    // succeeds. All of this happens inside a single call to
    // `next_bindings`, exactly like a real plan's context guard, which is
    // queried once per candidate plan (`ApplicablePlanSelection::next_plan`).
    #[test]
    fn rule_reevaluated_after_unrelated_backtracking() {
        let mut bb = KnowledgeBase::default();

        // alt(1). alt(2). -- two independent ways for the first operand of
        // the conjunction below to succeed, so the second solution requires
        // backtracking into it.
        bb.assert_no_event(belief("alt", vec![number(1.0)]));
        bb.assert_no_event(belief("alt", vec![number(2.0)]));

        // sib(a). matches(a). -- exactly one way to satisfy the rule below.
        bb.assert_no_event(belief("sib", vec![string("a")]));
        bb.assert_no_event(belief("matches", vec![string("a")]));

        // has_conflict :- sib(S) & matches(S).
        // green_safe   :- not has_conflict.
        //
        // green_safe has to be its own named rule, not `not has_conflict`
        // inlined directly into the outer conjunction: as the outer
        // conjunction's operand, green_safe's own belief iterator (the
        // lookup of green_safe's definition, exactly one entry) does get
        // reset between attempts, same as any operand past the first. But
        // that reset never reaches inside green_safe's own cached rule
        // body -- the `not has_conflict` leaf living in there is operand 0
        // of a single-operand conjunction, so nothing ever resets *it*, and
        // has_conflict's own staleness survives buried inside it.
        let s = variable();
        bb.assert_no_event(rule(
            "has_conflict",
            vec![],
            and(vec![
                literal("sib", vec![variable_term(&s)]),
                literal("matches", vec![variable_term(&s)]),
            ]),
        ));
        bb.assert_no_event(rule(
            "green_safe",
            vec![],
            not(literal("has_conflict", vec![])),
        ));

        // alt(X) & green_safe -- has_conflict is genuinely true (sib(a) &
        // matches(a) holds) and nothing retracts it, so green_safe should
        // fail for every alt(X), not just the first one tried.
        let x = variable();
        let formula = and(vec![
            literal("alt", vec![variable_term(&x)]),
            literal("green_safe", vec![]),
        ]);

        let mut query = (&formula).into_query(&bb);

        assert!(
            query.next_bindings(None).is_none(),
            "should fail for both alt(1) and alt(2): has_conflict is \
             genuinely true and nothing ever retracts it"
        );
    }

    #[test]
    fn rule_query_ignores_already_bound_argument_and_matches_unrelated_grounding() {
        let mut bb = KnowledgeBase::default();

        // down(X) :- requested(X).
        // requested(a). -- the only ground fact, for a DIFFERENT constant
        // than the one being asked about below.
        let x = variable();
        bb.assert_no_event(rule(
            "down",
            vec![variable_term(&x)],
            literal("requested", vec![variable_term(&x)]),
        ));
        bb.assert_no_event(belief("requested", vec![string("a")]));

        // Ground-argument sanity check: down("b") is unambiguously false
        // (nothing at all matches `requested(b)`), and the framework agrees.
        let ground_query = not(literal("down", vec![string("b")]));
        let mut ground = (&ground_query).into_query(&bb);
        assert!(
            ground.next_bindings(None).is_some(),
            "sanity check: down(b) should be false since requested(b) doesn't exist"
        );

        // Same question, but GW arrives pre-bound to "b" via a real
        // `Bindings` map (exactly how a variable bound by an earlier
        // conjunct, e.g. `gateway_b(GW) & not gateway_down(GW)`, is passed
        // into evaluating the second conjunct) instead of being written
        // directly into the literal as a ground string.
        let gw = variable();
        let b_term = string("b");
        let pre_bound = crate::testing::bindings(vec![(
            gw.clone(),
            crate::term::view::TermView::Term(&b_term),
        )]);
        let bound_query = not(literal("down", vec![variable_term(&gw)]));
        let mut bound = (&bound_query).into_query(&bb);

        assert!(
            bound.next_bindings(Some(&pre_bound)).is_some(),
            "down(GW) with GW pre-bound to \"b\" should also be false -- \
             identical question to the ground-literal case above, just \
             asked through a variable. If this fails, the rule resolved its \
             own head variable against `requested(a)` without checking that \
             GW was already constrained to \"b\", and wrongly rebound GW to \
             \"a\" instead of rejecting the mismatch."
        );
    }

    #[test]
    fn rule_query_preserves_unrelated_bindings_from_earlier_conjuncts() {
        let mut bb = KnowledgeBase::default();

        // down(X) :- requested(X).
        // requested(a).
        let x = variable();
        bb.assert_no_event(rule(
            "down",
            vec![variable_term(&x)],
            literal("requested", vec![variable_term(&x)]),
        ));
        bb.assert_no_event(belief("requested", vec![string("a")]));

        // echo(Y) & gateway_b(GW) bind Y and GW before the query reaches down(GW).
        bb.assert_no_event(belief("echo", vec![string("keep-me")]));
        bb.assert_no_event(belief("gateway_b", vec![string("a")]));

        let (y, gw) = (variable(), variable());
        let formula = and(vec![
            literal("echo", vec![variable_term(&y)]),
            literal("gateway_b", vec![variable_term(&gw)]),
            literal("down", vec![variable_term(&gw)]),
        ]);

        let mut query = (&formula).into_query(&bb);
        let bindings = query
            .next_bindings(None)
            .expect("down(GW) should succeed: GW = \"a\" matches requested(a)");

        assert_eq!(
            bindings.get_view(&y),
            Some(&string("keep-me").as_view()),
            "Y's binding from the first conjunct should survive the later rule \
             call to down(GW), even though `down` never mentions Y at all. If \
             this fails, `next_bindings_for_rule` dropped it again."
        );
    }

    // Reproduces microgrid's best_route/better_exists/reachable shape.
    // reachable(Dest, Via, Cost) has TWO separate clauses (direct + one
    // indirect hop), and better_exists(Dest, Cost) :- reachable(Dest, _,
    // C2) & C2 < Cost is invoked from inside a `not(...)` where Dest/Cost
    // are already bound by the outer rule's own first conjunct -- one level
    // deeper than the `down(GW)` case above (that rule's body never itself
    // called another named, multi-clause rule).
    //
    // This used to hang forever (pinned at ~100% CPU, confirmed 12+ minutes)
    // rather than return a wrong answer: `Cost` is a rule-head parameter
    // that the body itself needs (`C2 < Cost`), but the old
    // `next_bindings_for_rule` only unified the head against the caller's
    // literal *after* resolving the body once, so `Cost` was permanently
    // unbound while the body ran. Combined with `reachable`'s first clause
    // being a purely relational (unify-only) sub-rule with no belief
    // iterator to exhaust, backtracking into it after the doomed `C2 <
    // Cost` check kept re-deriving the exact same answer forever instead of
    // ever reporting "no more candidates". Fixed by (1) unifying the rule
    // head against the caller's literal *before* resolving the body, so
    // `Cost` is seeded in for the body to use, and (2) giving non-negated,
    // belief-iterator-less ground queries (bare relational formulas) a
    // single-shot "already evaluated" guard, mirroring the existing
    // negated-query guard.
    #[test]
    fn nested_rule_over_disjunctive_rule_still_leaks_unrelated_bindings() {
        let mut bb = KnowledgeBase::default();

        // reachable(load, load, 1).            -- direct link, cost 1
        // reachable(load, c, 14).               -- indirect via c, cost 14
        // reachable(load, source, 11).          -- indirect via source, cost 11
        let (d, v, c) = (variable(), variable(), variable());
        bb.assert_no_event(rule(
            "reachable",
            vec![variable_term(&d), variable_term(&v), variable_term(&c)],
            and(vec![
                unify(expr(variable_term(&d)), expr(string("load"))),
                unify(expr(variable_term(&v)), expr(string("load"))),
                unify(expr(variable_term(&c)), expr(number(1.0))),
            ]),
        ));
        let (d2, v2, c2) = (variable(), variable(), variable());
        bb.assert_no_event(rule(
            "reachable",
            vec![variable_term(&d2), variable_term(&v2), variable_term(&c2)],
            and(vec![
                unify(expr(variable_term(&d2)), expr(string("load"))),
                unify(expr(variable_term(&v2)), expr(string("c"))),
                unify(expr(variable_term(&c2)), expr(number(14.0))),
            ]),
        ));
        let (d3, v3, c3) = (variable(), variable(), variable());
        bb.assert_no_event(rule(
            "reachable",
            vec![variable_term(&d3), variable_term(&v3), variable_term(&c3)],
            and(vec![
                unify(expr(variable_term(&d3)), expr(string("load"))),
                unify(expr(variable_term(&v3)), expr(string("source"))),
                unify(expr(variable_term(&c3)), expr(number(11.0))),
            ]),
        ));

        // better_exists(Dest, Cost) :- reachable(Dest, _, C2) & C2 < Cost.
        let (bd, bc, bc2, b_via) = (variable(), variable(), variable(), variable());
        bb.assert_no_event(rule(
            "better_exists",
            vec![variable_term(&bd), variable_term(&bc)],
            and(vec![
                literal(
                    "reachable",
                    vec![
                        variable_term(&bd),
                        variable_term(&b_via),
                        variable_term(&bc2),
                    ],
                ),
                cmp(
                    expr(variable_term(&bc2)),
                    CompareOperator::LessThan,
                    false,
                    expr(variable_term(&bc)),
                ),
            ]),
        ));

        // best_route(Dest, Via, Cost) :- reachable(Dest, Via, Cost) & not better_exists(Dest, Cost).
        let (rd, rv, rc) = (variable(), variable(), variable());
        bb.assert_no_event(rule(
            "best_route",
            vec![variable_term(&rd), variable_term(&rv), variable_term(&rc)],
            and(vec![
                literal(
                    "reachable",
                    vec![variable_term(&rd), variable_term(&rv), variable_term(&rc)],
                ),
                not(literal(
                    "better_exists",
                    vec![variable_term(&rd), variable_term(&rc)],
                )),
            ]),
        ));

        // Enumerate every best_route(load, Via, Cost) solution, exactly what
        // `.forall(best_route(Dest, Via, Cost) & ..., sync_route(...))` does
        // in the real agent. Only (via=load, cost=1) should ever satisfy
        // "not better_exists" -- the other two candidates both have a
        // strictly cheaper alternative (cost 1) and should be rejected.
        let (via, cost) = (variable(), variable());
        let formula = literal(
            "best_route",
            vec![string("load"), variable_term(&via), variable_term(&cost)],
        );
        let mut query = (&formula).into_query(&bb);

        let mut solutions = alloc::vec::Vec::new();
        while let Some(bindings) = query.next_bindings(None) {
            solutions.push((
                bindings.get_view(&via).cloned(),
                bindings.get_view(&cost).cloned(),
            ));
        }

        assert_eq!(
            solutions,
            alloc::vec![(
                Some(string("load").as_view()),
                Some(TermView::Number(1.0.into()))
            )],
            "best_route(load, Via, Cost) should have exactly one solution \
             (via=load, cost=1) -- if this contains the via=c/cost=14 or \
             via=source/cost=11 candidates too, better_exists(load, 14) / \
             better_exists(load, 11) both wrongly evaluated false instead of \
             true, i.e. `not(...)`'s nested rule call failed to see that a \
             cheaper reachable(load, _, _) exists once the NAF'd rule itself \
             calls a second, disjunctive (multi-clause) rule."
        );
    }

    #[test]
    fn single_solution_conjunct_before_enumerating_conjunct_yields_all_combinations() {
        // Regression: Conjunction::next_bindings used to restart its
        // cartesian-product walk at operand 0 on every call. If operand 0 has
        // exactly one solution, it was left exhausted and unreset after the
        // first success, so the second call bailed immediately instead of
        // asking operand 1 (neighbor) for its remaining solutions.
        let mut bb = KnowledgeBase::default();
        bb.assert_no_event(belief("node_active", vec![]));
        bb.assert_no_event(belief("neighbor", vec![string("n1")]));
        bb.assert_no_event(belief("neighbor", vec![string("n2")]));

        let via = variable();
        let formula = and(vec![
            literal("node_active", vec![]),
            literal("neighbor", vec![variable_term(&via)]),
        ]);

        let mut query = (&formula).into_query(&bb);

        let first = query
            .next_bindings(None)
            .expect("should find first neighbor");
        assert_eq!(first.get_view(&via), Some(&string("n1").as_view()));

        let second = query
            .next_bindings(None)
            .expect("should find second neighbor too");
        assert_eq!(second.get_view(&via), Some(&string("n2").as_view()));

        assert!(query.next_bindings(None).is_none());
    }

    #[test]
    fn negating_conjunction_with_shared_variable_checks_the_whole_conjunction() {
        // Regression: negated conjunctions used to be decomposed via De
        // Morgan into independently negated leaves, which is unsound once
        // the leaves share a variable. The relational leaf ended up
        // evaluated with that variable permanently unbound, its evaluation
        // error silently read as "false", and the negation vacuously
        // succeeded no matter what via_candidate held.
        let mut bb = KnowledgeBase::default();
        bb.assert_no_event(belief("via_candidate", vec![string("n1"), number(1.0)]));
        bb.assert_no_event(belief("via_candidate", vec![string("n5"), number(5.0)]));

        let other_id = variable();
        let formula = not(and(vec![
            literal(
                "via_candidate",
                vec![variable_term(&variable()), variable_term(&other_id)],
            ),
            cmp(
                expr(variable_term(&other_id)),
                CompareOperator::LessThan,
                false,
                expr(number(2.0)),
            ),
        ]));

        let mut query = (&formula).into_query(&bb);
        // A via_candidate with id 1.0 < 2.0 exists, so the negation must fail.
        assert!(query.next_bindings(None).is_none());
    }

    #[test]
    fn double_negation_behaves_like_the_original() {
        // `not (not p)` no longer collapses structurally, it lowers to a
        // negated subquery wrapping a negated leaf. Confirms that still
        // evaluates the same as the original, unnegated formula.
        let mut bb = KnowledgeBase::default();
        bb.assert_no_event(belief("p", vec![]));

        let formula = not(not(literal("p", vec![])));
        assert!((&formula).into_query(&bb).next_bindings(None).is_some());

        let formula = not(not(literal("q", vec![])));
        assert!((&formula).into_query(&bb).next_bindings(None).is_none());
    }

    #[test]
    fn disjunction_nested_inside_conjunction_resolves_via_subquery() {
        // Confirms the direct-mapping lowering handles a compound formula in
        // leaf position (here, an OR inside an AND) as a subquery, and still
        // enumerates every combination correctly.
        let mut bb = KnowledgeBase::default();
        bb.assert_no_event(belief("node_active", vec![]));
        bb.assert_no_event(belief("neighbor", vec![string("n1")]));
        bb.assert_no_event(belief("neighbor", vec![string("n2")]));

        let via = variable();
        let formula = and(vec![
            literal("node_active", vec![]),
            or(vec![literal("neighbor", vec![variable_term(&via)])]),
        ]);

        let mut query = (&formula).into_query(&bb);

        let first = query
            .next_bindings(None)
            .expect("should find first neighbor");
        assert_eq!(first.get_view(&via), Some(&string("n1").as_view()));

        let second = query
            .next_bindings(None)
            .expect("should find second neighbor too");
        assert_eq!(second.get_view(&via), Some(&string("n2").as_view()));

        assert!(query.next_bindings(None).is_none());
    }
}
