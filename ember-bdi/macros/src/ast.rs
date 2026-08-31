use std::collections::HashMap;

use proc_macro2::{Ident, TokenStream};
use quote::{ToTokens, format_ident, quote};

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
pub(crate) struct Belief(pub(crate) Literal, pub(crate) Option<LogicalExpression>);

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
    List(Box<[Term]>),
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
    And(Box<(LogicalExpression, LogicalExpression)>),
    Or(Box<(LogicalExpression, LogicalExpression)>),
}

#[derive(Debug, Clone)]
pub(crate) enum SimpleLogicalExpression {
    Literal(Literal),
    Rel(RelationalExpression),
    /// A pure built-in action (`.now`, `.me`, `.append`, `.member`, `.findall`, `.min`, `.max`)
    /// used in a context guard.
    Action(PureAction),
    Not(Box<SimpleLogicalExpression>),
    Group(Box<LogicalExpression>),
}

#[derive(Debug, Clone)]
pub(crate) enum PureAction {
    Now(Variable),
    Me(Variable),
    /// Binds `List` with `Item` appended to the given variable.
    Append(ListOrVariable, Term, Variable),
    /// Whether `Item` structurally equals some element of `List`.
    Member(Term, ListOrVariable),
    /// Binds `List` to every instantiation of `Template` that satisfies `Query`.
    Findall(Term, Box<LogicalExpression>, Variable),
    /// Checks `Minimal` against, or binds it to, the smallest element of `List`.
    Min(ListOrVariable, Term),
    /// Checks `Maximal` against, or binds it to, the largest element of `List`.
    Max(ListOrVariable, Term),
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
    Belief {
        trigger: BodyFormulaBeliefTrigger,
        literal: Literal,
        silent: bool,
    },
    Goal {
        trigger: BodyFormulaGoalTrigger,
        literal: Literal,
    },
    Unify {
        lhs: RelationalTerm,
        rhs: RelationalTerm,
    },
    Action(Spanned<Action>),
}

#[derive(Debug, Clone)]
pub(crate) enum BodyFormulaBeliefTrigger {
    Add,
    Remove,
}

#[derive(Debug, Clone)]
pub(crate) enum BodyFormulaGoalTrigger {
    Achieve,
    Query,
}

#[derive(Debug, Clone)]
pub(crate) enum Action {
    Builtin(BuiltinAction),
    User(Spanned<AtomicFormula>),
}

/// Which `.send(...)` outcome a callback fires on, see `BuiltinAction::Send::callbacks`.
#[derive(Debug, Clone, Copy)]
#[allow(clippy::enum_variant_names)]
pub(crate) enum CallbackKind {
    OnSuccess,
    OnRetry,
    OnFailure,
    OnComplete,
}

#[derive(Debug, Clone)]
pub enum BuiltinAction {
    Pure(PureAction),
    Impure(ImpureAction),
}

#[derive(Debug, Clone)]
pub enum ImpureAction {
    Log(String, Box<[Term]>),
    StopPlatform,
    Send {
        aid: AidOrVariable,
        trigger: Trigger,
        literal: LiteralOrVariable,
        /// `[on_failure(retry(N)), ...]`, fired by the runtime `SendCallbacks` hook, not called directly.
        callbacks: Box<[(CallbackKind, Literal)]>,
    },
    Wait {
        interval_millis: u64,
    },
    Forall {
        query: LogicalExpression,
        goal: LiteralOrVariable,
    },
    At {
        delay_millis: u64,
        goal: Literal,
    },
}

#[derive(Debug, Clone)]
pub enum AidOrVariable {
    Aid {
        aid_name: String,
        aid_platform: Option<String>,
    },
    Variable(Variable),
}

#[derive(Debug, Clone)]
pub enum LiteralOrVariable {
    Literal(Literal),
    Variable(Variable),
}

#[derive(Debug, Clone)]
pub(crate) enum ListOrVariable {
    List(Box<[Term]>),
    Variable(Variable),
}

pub(crate) struct AstVisitor {
    /// Maps from a `Variable` name to the ident name of the generated rust variable.
    pub(crate) variable_map: HashMap<String, Ident>,
    /// Keeps track of the amount of anonymous variables used to generate a unique name for each
    /// one.
    pub(crate) anonymous_variable_count: usize,
    pub(crate) agent_ident: Ident,
}

impl AstVisitor {
    pub(crate) fn new(agent_ident: Ident) -> Self {
        Self {
            variable_map: HashMap::new(),
            anonymous_variable_count: 0,
            agent_ident,
        }
    }

    pub(crate) fn visit_belief(&mut self, Belief(literal, rule): &Belief) -> impl ToTokens {
        let literal = self.visit_literal(literal).into_token_stream();
        match rule {
            Some(rule) => {
                let rule = self.visit_logical_expression(rule);
                quote! {
                    ::ember::agent::bdi::knowledge::belief::Knowledge::from(
                        (#literal, #rule)
                    )
                }
            }
            None => quote! {
                ::ember::agent::bdi::knowledge::belief::Knowledge::from(
                    #literal
                )
            },
        }
    }

    pub(crate) fn visit_goal(&mut self, Goal(literal): &Goal) -> impl ToTokens {
        self.visit_literal(literal)
    }

    pub(crate) fn visit_plan(
        &mut self,
        Plan {
            event,
            context,
            body,
        }: &Plan,
    ) -> impl ToTokens {
        let trigger = self.visit_triggering_event(event).into_token_stream();
        let context = match context {
            Some(c) => {
                let context = self.visit_context(c);
                quote! {
                    ::core::option::Option::Some(#context)
                }
            }
            None => quote! { ::core::option::Option::None },
        };
        let body = self.visit_body(body);

        quote! {
            ::ember::agent::bdi::plan::Plan {
                trigger: #trigger,
                context: #context,
                body: #body,
            }
        }
    }

    fn visit_context(&mut self, Context(expression): &Context) -> impl ToTokens {
        self.visit_logical_expression(expression)
    }

    fn visit_body(&mut self, Body(formulae): &Body) -> impl ToTokens {
        let formulae = formulae
            .into_iter()
            .map(|f| self.visit_body_formula(f).into_token_stream());

        quote! {
            ::alloc::rc::Rc::new([
                #(#formulae),*
            ])
        }
    }

    fn visit_body_formula(&mut self, formula: &BodyFormula) -> impl ToTokens {
        match formula {
            BodyFormula::Belief {
                trigger,
                literal,
                silent,
            } => {
                let literal = self.visit_literal(literal);
                match trigger {
                    BodyFormulaBeliefTrigger::Add => quote! {
                        ::ember::agent::bdi::plan::Formula::Belief {
                            trigger: ::ember::agent::bdi::event::Trigger::Addition,
                            belief: #literal,
                            silent: #silent,
                        }
                    },
                    BodyFormulaBeliefTrigger::Remove => quote! {
                        ::ember::agent::bdi::plan::Formula::Belief {
                            trigger: ::ember::agent::bdi::event::Trigger::Deletion,
                            belief: #literal,
                            silent: #silent,
                        }
                    },
                }
            }
            BodyFormula::Goal { trigger, literal } => {
                let literal = self.visit_literal(literal);
                match trigger {
                    BodyFormulaGoalTrigger::Achieve => quote! {
                        ::ember::agent::bdi::plan::Formula::Goal {
                            kind: ::ember::agent::bdi::event::GoalKind::Achieve,
                            goal: #literal,
                        }
                    },
                    BodyFormulaGoalTrigger::Query => quote! {
                        ::ember::agent::bdi::plan::Formula::Goal {
                            kind: ::ember::agent::bdi::event::GoalKind::Query,
                            goal: #literal,
                        }
                    },
                }
            }
            BodyFormula::Unify { lhs, rhs } => {
                let lhs = self.visit_relational_term(lhs).into_token_stream();
                let rhs = self.visit_relational_term(rhs).into_token_stream();
                quote! {
                    ::ember::agent::bdi::plan::Formula::Unify { lhs: #lhs, rhs: #rhs }
                }
            }
            BodyFormula::Action(action) => {
                let action = self.visit_action(action);
                quote! {
                    ::ember::agent::bdi::plan::Formula::Action(#action)
                }
            }
        }
    }

    fn visit_logical_expression(&mut self, expression: &LogicalExpression) -> impl ToTokens {
        match expression {
            LogicalExpression::Simple(expression) => {
                let expression = self.visit_simple_logical_expression(expression);
                quote! { #expression }
            }
            LogicalExpression::And(operands) => {
                let (lhs, rhs) = operands.as_ref();
                let lhs = self.visit_logical_expression(lhs).into_token_stream();
                let rhs = self.visit_logical_expression(rhs);

                quote! {
                    ::ember::agent::bdi::plan::QueryFormula::Logical {
                        operator: ::ember::agent::bdi::plan::LogicalOperator::Conjunction,
                        operands: ::alloc::boxed::Box::new([#lhs, #rhs]),
                    }
                }
            }
            LogicalExpression::Or(operands) => {
                let (lhs, rhs) = operands.as_ref();
                let lhs = self.visit_logical_expression(lhs).into_token_stream();
                let rhs = self.visit_logical_expression(rhs);

                quote! {
                    ::ember::agent::bdi::plan::QueryFormula::Logical {
                        operator: ::ember::agent::bdi::plan::LogicalOperator::Disjunction,
                        operands: ::alloc::boxed::Box::new([#lhs, #rhs]),
                    }
                }
            }
        }
    }

    fn visit_simple_logical_expression(
        &mut self,
        expression: &SimpleLogicalExpression,
    ) -> impl ToTokens {
        match expression {
            SimpleLogicalExpression::Literal(literal) => {
                let literal = self.visit_literal(literal);
                quote! { ::ember::agent::bdi::plan::QueryFormula::Literal(#literal) }
            }
            SimpleLogicalExpression::Rel(expression) => {
                let expression = self.visit_relational_expression(expression);
                quote! { ::ember::agent::bdi::plan::QueryFormula::Relational(#expression) }
            }
            SimpleLogicalExpression::Action(action) => {
                let action = self.visit_pure_action(action);
                quote! { ::ember::agent::bdi::plan::QueryFormula::Action(#action) }
            }
            SimpleLogicalExpression::Not(expression) => {
                let expression = self.visit_simple_logical_expression(expression);
                quote! { ::ember::agent::bdi::plan::QueryFormula::Not(::alloc::boxed::Box::new(#expression)) }
            }
            SimpleLogicalExpression::Group(expression) => {
                let expression = self.visit_logical_expression(expression);
                quote! { #expression }
            }
        }
    }

    fn visit_relational_expression(
        &mut self,
        RelationalExpression {
            operator,
            operands: (lhs, rhs),
        }: &RelationalExpression,
    ) -> impl ToTokens {
        let (lhs, rhs) = (
            self.visit_relational_term(lhs).into_token_stream(),
            self.visit_relational_term(rhs).into_token_stream(),
        );
        quote! {
            ::ember::agent::bdi::plan::RelationalQueryFormula {
                operator: #operator,
                operands: (#lhs, #rhs),
            }
        }
    }

    fn visit_relational_term(&mut self, term: &RelationalTerm) -> impl ToTokens {
        match term {
            RelationalTerm::Literal(literal) => {
                let literal = self.visit_literal(literal);
                quote! {
                    ::ember::agent::bdi::plan::ArithmeticExpression::Term(
                        ::ember::agent::bdi::term::owned::Term::Literal(#literal),
                    )
                }
            }
            RelationalTerm::Arithm(expression) => self
                .visit_arithmetic_expression(expression)
                .into_token_stream(),
        }
    }

    fn visit_arithmetic_expression(
        &mut self,
        ArithmeticExpression { lhs, rhs }: &ArithmeticExpression,
    ) -> impl ToTokens {
        let lhs = self.visit_arithmetic_term(lhs).into_token_stream();
        rhs.as_ref().map_or_else(
            {
                let lhs = lhs.clone();
                || lhs
            },
            |(operator, rhs)| {
                let rhs = self.visit_arithmetic_expression(rhs);
                quote! {
                    ::ember::agent::bdi::plan::ArithmeticExpression::Operation {
                        operator: #operator,
                        operands: ::alloc::boxed::Box::new([#lhs, #rhs]),
                    }
                }
            },
        )
    }

    fn visit_arithmetic_term(
        &mut self,
        ArithmeticTerm { lhs, rhs }: &ArithmeticTerm,
    ) -> impl ToTokens {
        let lhs = self.visit_arithmetic_factor(lhs).into_token_stream();
        rhs.as_ref().map_or_else(
            {
                let lhs = lhs.clone();
                || lhs
            },
            |(operator, rhs)| {
                let rhs = self.visit_arithmetic_term(rhs);
                quote! {
                    ::ember::agent::bdi::plan::ArithmeticExpression::Operation {
                        operator: #operator,
                        operands: ::alloc::boxed::Box::new([#lhs, #rhs]),
                    }
                }
            },
        )
    }

    fn visit_arithmetic_factor(&mut self, factor: &ArithmeticFactor) -> impl ToTokens {
        match factor {
            ArithmeticFactor::Number(number) => quote! {
                ::ember::agent::bdi::plan::ArithmeticExpression::Term(
                    ::ember::agent::bdi::term::owned::Term::Number(#number.into())
                )
            },
            ArithmeticFactor::Variable(variable) => {
                let variable = self.visit_variable(variable);
                quote! {
                    ::ember::agent::bdi::plan::ArithmeticExpression::Term(
                        ::ember::agent::bdi::term::owned::Term::Variable(#variable)
                    )
                }
            }
            ArithmeticFactor::Neg(factor) => {
                let factor = self.visit_arithmetic_factor(factor);
                quote! {
                    ::ember::agent::bdi::plan::ArithmeticExpression::Operation {
                        operator: ::ember::agent::bdi::plan::ArithmeticOperator::Min,
                        operands: ::alloc::boxed::Box::new([#factor]),
                    }
                }
            }
            ArithmeticFactor::Group(expression) => self
                .visit_arithmetic_expression(expression)
                .into_token_stream(),
        }
    }

    fn visit_literal(&mut self, Literal { negated, formula }: &Literal) -> impl ToTokens {
        let structure = self.visit_atomic_formula(formula);
        quote! {
            ::ember::agent::bdi::literal::Literal {
                negated: #negated,
                structure: #structure
            }
        }
    }

    fn visit_atomic_formula(
        &mut self,
        AtomicFormula { functor, arguments }: &AtomicFormula,
    ) -> impl ToTokens {
        let arguments = match arguments {
            Some(args) => {
                let args = args
                    .into_iter()
                    .map(|a| self.visit_term(a).into_token_stream())
                    .collect::<Vec<_>>();
                quote! {
                    ::core::option::Option::Some(::alloc::boxed::Box::new([
                        #(#args),*
                    ]))
                }
            }
            None => quote! { ::core::option::Option::None },
        };
        quote! {
            ::ember::agent::bdi::term::Structure {
                functor: #functor,
                arguments: #arguments,
            }
        }
    }

    fn visit_term(&mut self, term: &Term) -> impl ToTokens {
        match term {
            Term::Literal(literal) => {
                let literal = self.visit_literal(literal);

                quote! {
                    ::ember::agent::bdi::term::owned::Term::Literal(#literal)
                }
            }
            Term::Variable(variable) => {
                let variable = self.visit_variable(variable);
                quote! {
                    ::ember::agent::bdi::term::owned::Term::Variable(#variable)
                }
            }
            Term::Number(number) => quote! {
                ::ember::agent::bdi::term::owned::Term::Number(#number.into())
            },
            Term::String(string) => {
                quote! {
                    ::ember::agent::bdi::term::owned::Term::String(#string.into())
                }
            }
            Term::List(items) => {
                let items = items
                    .iter()
                    .map(|t| self.visit_term(t).into_token_stream())
                    .collect::<Vec<_>>();
                quote! {
                    ::ember::agent::bdi::term::owned::Term::List(::alloc::boxed::Box::new([
                        #(#items),*
                    ]))
                }
            }
        }
    }

    fn visit_variable(&mut self, Variable(name): &Variable) -> impl ToTokens {
        let var_name = match name.as_str() {
            "_" => {
                let anon_name = format!("var_anon_{}", {
                    let count = self.anonymous_variable_count;
                    self.anonymous_variable_count += 1;
                    count
                });
                self.variable_map
                    .entry(anon_name.clone())
                    .or_insert_with(|| format_ident!("{}", anon_name))
            }
            _ => self
                .variable_map
                .entry(name.clone())
                .or_insert_with(|| format_ident!("var_{}", name.to_lowercase())),
        };
        quote! {
            #var_name.clone()
        }
    }

    fn visit_triggering_event(
        &mut self,
        TriggeringEvent {
            trigger,
            goal,
            event,
        }: &TriggeringEvent,
    ) -> impl ToTokens {
        let goal = match goal {
            Some(goal) => quote! { ::core::option::Option::Some(#goal) },
            None => quote! { ::core::option::Option::None },
        };
        let event = self.visit_literal(event);

        quote! {
            ::ember::agent::bdi::event::TriggeringEvent {
                trigger: #trigger,
                goal: #goal,
                event: #event,
            }
        }
    }

    fn visit_action(&mut self, action: &Action) -> impl ToTokens {
        match action {
            Action::Builtin(action) => {
                let action = self.visit_builtin_action(action);
                quote! {
                    ::ember::agent::bdi::plan::action::Action::Builtin(#action)
                }
            }
            Action::User(crate::ast::Spanned {
                node: AtomicFormula { functor, arguments },
                ..
            }) => {
                let factory_ident = format_ident!("{}_action", functor.0.as_str());
                let args_tokens = match arguments {
                    Some(args) => {
                        let args = args
                            .into_iter()
                            .map(|a| self.visit_term(a).into_token_stream())
                            .collect::<Vec<_>>();
                        quote! { #(#args),* }
                    }
                    None => quote! {},
                };
                let agent_ident = &self.agent_ident;
                quote! {
                    ::ember::agent::bdi::plan::action::Action::User(#agent_ident::#factory_ident(#args_tokens))
                }
            }
        }
    }

    fn visit_builtin_action(&mut self, action: &BuiltinAction) -> impl ToTokens {
        match action {
            BuiltinAction::Pure(action) => {
                let action = self.visit_pure_action(action);
                quote! {
                    ::ember::agent::bdi::plan::action::BuiltinAction::Pure(#action)
                }
            }
            // `WaitState`/`AtState` have private fields, so the runtime only lets us build these
            // through `BuiltinAction::wait`/`::at`, which already return the wrapped
            // `Impure(...)` value, unlike every other `ImpureAction` variant.
            BuiltinAction::Impure(ImpureAction::Wait { interval_millis }) => quote! {
                ::ember::agent::bdi::plan::action::BuiltinAction::wait(::core::time::Duration::from_millis(#interval_millis))
            },
            BuiltinAction::Impure(ImpureAction::At { delay_millis, goal }) => {
                let goal = self.visit_literal(goal).into_token_stream();
                quote! {
                    ::ember::agent::bdi::plan::action::BuiltinAction::at(
                        ::core::time::Duration::from_millis(#delay_millis),
                        #goal,
                    )
                }
            }
            BuiltinAction::Impure(action) => {
                let action = self.visit_impure_action(action);
                quote! {
                    ::ember::agent::bdi::plan::action::BuiltinAction::Impure(#action)
                }
            }
        }
    }

    fn visit_pure_action(&mut self, action: &PureAction) -> impl ToTokens {
        match action {
            PureAction::Now(variable) => {
                let variable = self.visit_variable(variable).into_token_stream();
                quote! {
                    ::ember::agent::bdi::plan::action::PureAction::Now(#variable)
                }
            }
            PureAction::Me(variable) => {
                let variable = self.visit_variable(variable).into_token_stream();
                quote! {
                    ::ember::agent::bdi::plan::action::PureAction::Me(#variable)
                }
            }
            PureAction::Append(list, item, variable) => {
                let list = self.visit_list_or_variable(list).into_token_stream();
                let item = self.visit_term(item).into_token_stream();
                let variable = self.visit_variable(variable).into_token_stream();
                quote! {
                    ::ember::agent::bdi::plan::action::PureAction::Append(#list, #item, #variable)
                }
            }
            PureAction::Member(item, list) => {
                let item = self.visit_term(item).into_token_stream();
                let list = self.visit_list_or_variable(list).into_token_stream();
                quote! {
                    ::ember::agent::bdi::plan::action::PureAction::Member(#item, #list)
                }
            }
            PureAction::Findall(template, query, variable) => {
                let template = self.visit_term(template).into_token_stream();
                let query = self.visit_logical_expression(query).into_token_stream();
                let query = quote! { ::alloc::boxed::Box::new(#query) };
                let variable = self.visit_variable(variable).into_token_stream();
                quote! {
                    ::ember::agent::bdi::plan::action::PureAction::Findall(#template, #query, #variable)
                }
            }
            PureAction::Min(list, minimal) => {
                let list = self.visit_list_or_variable(list).into_token_stream();
                let minimal = self.visit_term(minimal).into_token_stream();
                quote! {
                    ::ember::agent::bdi::plan::action::PureAction::Min(#list, #minimal)
                }
            }
            PureAction::Max(list, maximal) => {
                let list = self.visit_list_or_variable(list).into_token_stream();
                let maximal = self.visit_term(maximal).into_token_stream();
                quote! {
                    ::ember::agent::bdi::plan::action::PureAction::Max(#list, #maximal)
                }
            }
        }
    }

    fn visit_impure_action(&mut self, action: &ImpureAction) -> impl ToTokens {
        match action {
            ImpureAction::Log(level, terms) => {
                let terms = terms
                    .into_iter()
                    .map(|t| self.visit_term(t).to_token_stream());
                quote! {
                    ::ember::agent::bdi::plan::action::ImpureAction::Log(
                        #level.parse().expect("failed to parse log level"),
                        ::alloc::boxed::Box::new([#(#terms),*])
                    )
                }
            }
            ImpureAction::StopPlatform => {
                quote! {
                    ::ember::agent::bdi::plan::action::ImpureAction::StopPlatform
                }
            }
            ImpureAction::Send {
                aid,
                trigger,
                literal,
                callbacks,
            } => {
                let aid = self.visit_aid_or_variable(aid).into_token_stream();
                let trigger_ts = match trigger {
                    Trigger::Addition => quote! {
                        ::ember::agent::bdi::event::Trigger::Addition
                    },
                    Trigger::Deletion => quote! {
                        ::ember::agent::bdi::event::Trigger::Deletion
                    },
                };
                let literal_ts = self.visit_literal_or_variable(literal).into_token_stream();
                let callbacks_ts = callbacks.iter().map(|(kind, goal)| {
                    let kind_ts = match kind {
                        CallbackKind::OnSuccess => quote! {
                            ::ember::agent::bdi::plan::action::CallbackKind::OnSuccess
                        },
                        CallbackKind::OnRetry => quote! {
                            ::ember::agent::bdi::plan::action::CallbackKind::OnRetry
                        },
                        CallbackKind::OnFailure => quote! {
                            ::ember::agent::bdi::plan::action::CallbackKind::OnFailure
                        },
                        CallbackKind::OnComplete => quote! {
                            ::ember::agent::bdi::plan::action::CallbackKind::OnComplete
                        },
                    };
                    let goal_ts = self.visit_literal(goal).into_token_stream();
                    quote! {
                        (
                            #kind_ts,
                            ::ember::agent::bdi::term::owned::composite::VariableOrLiteral::Literal(#goal_ts),
                        )
                    }
                });
                quote! {
                    ::ember::agent::bdi::plan::action::ImpureAction::SendLiteral(
                        #aid,
                        #trigger_ts,
                        #literal_ts,
                        ::alloc::boxed::Box::new([#(#callbacks_ts),*]),
                    )
                }
            }
            ImpureAction::Forall { query, goal } => {
                let query = self.visit_logical_expression(query).into_token_stream();
                let goal = self.visit_literal_or_variable(goal).into_token_stream();
                quote! {
                    ::ember::agent::bdi::plan::action::ImpureAction::Forall {
                        query: #query,
                        goal: #goal,
                    }
                }
            }
            ImpureAction::Wait { .. } | ImpureAction::At { .. } => {
                unreachable!("handled directly in visit_builtin_action")
            }
        }
    }

    fn visit_aid_or_variable(&mut self, aid: &AidOrVariable) -> impl ToTokens {
        match aid {
            AidOrVariable::Aid {
                aid_name,
                aid_platform,
            } => {
                let aid = match aid_platform {
                    None => quote! { ::ember::agent::Aid::local(#aid_name) },
                    Some(p) => quote! { ::ember::agent::Aid::general(#aid_name, #p) },
                };
                quote! {
                    ::ember::agent::bdi::term::owned::composite::VariableOrReceiver::Receiver(
                        ::ember::message::Receiver::from(#aid)
                    )
                }
            }
            AidOrVariable::Variable(v) => {
                let variable = self.visit_variable(v);
                quote! {
                    ::ember::agent::bdi::term::owned::composite::VariableOrReceiver::Variable(#variable)
                }
            }
        }
    }

    fn visit_literal_or_variable(&mut self, literal: &LiteralOrVariable) -> impl ToTokens {
        match literal {
            LiteralOrVariable::Literal(literal) => {
                let literal = self.visit_literal(literal);
                quote! {
                    ::ember::agent::bdi::term::owned::composite::VariableOrLiteral::Literal(#literal)
                }
            }
            LiteralOrVariable::Variable(v) => {
                let variable = self.visit_variable(v);
                quote! {
                    ::ember::agent::bdi::term::owned::composite::VariableOrLiteral::Variable(#variable)
                }
            }
        }
    }

    fn visit_list_or_variable(&mut self, list: &ListOrVariable) -> impl ToTokens {
        match list {
            ListOrVariable::List(items) => {
                let items = items
                    .iter()
                    .map(|t| self.visit_term(t).into_token_stream())
                    .collect::<Vec<_>>();
                quote! {
                    ::ember::agent::bdi::term::owned::composite::VariableOrList::List(
                        ::alloc::boxed::Box::new([#(#items),*])
                    )
                }
            }
            ListOrVariable::Variable(v) => {
                let variable = self.visit_variable(v);
                quote! {
                    ::ember::agent::bdi::term::owned::composite::VariableOrList::Variable(#variable)
                }
            }
        }
    }
}

impl ToTokens for Atom {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self(value) = self;
        tokens.extend(quote! {
            ::ember::agent::bdi::term::owned::Atom(#value.into())
        });
    }
}

impl ToTokens for Trigger {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.extend(match self {
            Trigger::Addition => quote! { ::ember::agent::bdi::event::Trigger::Addition },
            Trigger::Deletion => quote! { ::ember::agent::bdi::event::Trigger::Deletion },
        });
    }
}

impl ToTokens for EventGoal {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.extend(match self {
            EventGoal::Achieve => quote! { ::ember::agent::bdi::event::GoalKind::Achieve },
            EventGoal::Query => quote! { ::ember::agent::bdi::event::GoalKind::Query },
        });
    }
}

impl ToTokens for RelationalOperator {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.extend(match self {
            RelationalOperator::Smaller => quote! {
                ::ember::agent::bdi::plan::RelationalOperator::Compare {
                    operator: ::ember::agent::bdi::plan::CompareOperator::LessThan,
                    equal: false,
                }
            },
            RelationalOperator::Larger => quote! {
                ::ember::agent::bdi::plan::RelationalOperator::Compare {
                    operator: ::ember::agent::bdi::plan::CompareOperator::GreaterThan,
                    equal: false,
                }
            },
            RelationalOperator::SmallerEq => quote! {
                ::ember::agent::bdi::plan::RelationalOperator::Compare {
                    operator: ::ember::agent::bdi::plan::CompareOperator::LessThan,
                    equal: true,
                }
            },
            RelationalOperator::LargerEq => quote! {
                ::ember::agent::bdi::plan::RelationalOperator::Compare {
                    operator: ::ember::agent::bdi::plan::CompareOperator::GreaterThan,
                    equal: true,
                }
            },
            RelationalOperator::Equal => quote! {
                ::ember::agent::bdi::plan::RelationalOperator::Compare {
                    operator: ::ember::agent::bdi::plan::CompareOperator::EqualTo,
                    equal: true,
                }
            },
            RelationalOperator::NotEqual => quote! {
                ::ember::agent::bdi::plan::RelationalOperator::Compare {
                    operator: ::ember::agent::bdi::plan::CompareOperator::EqualTo,
                    equal: false,
                }
            },
            RelationalOperator::Unify => quote! {
                ::ember::agent::bdi::plan::RelationalOperator::Unify
            },
        });
    }
}

impl ToTokens for PlusMin {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.extend(match self {
            PlusMin::Plus => quote! { ::ember::agent::bdi::plan::ArithmeticOperator::Sum },
            PlusMin::Min => quote! { ::ember::agent::bdi::plan::ArithmeticOperator::Min },
        });
    }
}

impl ToTokens for DivMul {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.extend(match self {
            DivMul::Division => quote! { ::ember::agent::bdi::plan::ArithmeticOperator::Div },
            DivMul::Multiplication => quote! { ::ember::agent::bdi::plan::ArithmeticOperator::Mul },
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn visit(visitor: &mut AstVisitor, name: &str) -> String {
        visitor
            .visit_variable(&Variable(name.into()))
            .into_token_stream()
            .to_string()
    }

    #[test]
    fn named_variable_resolves_to_the_same_rust_variable_on_every_occurrence() {
        let mut visitor = AstVisitor::new(format_ident!("Agent"));

        assert_eq!(visit(&mut visitor, "X"), visit(&mut visitor, "X"));
    }

    #[test]
    fn anonymous_variable_resolves_to_a_fresh_rust_variable_on_every_occurrence() {
        let mut visitor = AstVisitor::new(format_ident!("Agent"));

        assert_ne!(visit(&mut visitor, "_"), visit(&mut visitor, "_"));
    }

    #[test]
    fn anonymous_variable_does_not_collide_with_a_named_variable() {
        let mut visitor = AstVisitor::new(format_ident!("Agent"));

        assert_ne!(visit(&mut visitor, "_"), visit(&mut visitor, "X"));
    }
}
