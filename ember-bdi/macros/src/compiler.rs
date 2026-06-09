use std::collections::HashMap;

use proc_macro2::{Ident, TokenStream};
use quote::{ToTokens, format_ident, quote};

use crate::action::SystemAction;
use crate::ast::*;

pub(crate) mod actions;
pub(crate) mod asl;

struct AstVisitor {
    /// Maps from a `Variable` name to the ident name of the generated rust variable.
    variable_map: HashMap<String, Ident>,
    agent_ident: Ident,
}

impl AstVisitor {
    fn new(agent_ident: Ident) -> Self {
        Self {
            variable_map: HashMap::new(),
            agent_ident,
        }
    }
    fn visit_belief(&mut self, Belief(literal): &Belief) -> impl ToTokens {
        let literal = self.visit_literal(literal);
        quote! {
            ::ember::agent::bdi::knowledge::belief::Belief::from(
                #literal
            )
        }
    }

    fn visit_goal(&mut self, Goal(literal): &Goal) -> impl ToTokens {
        self.visit_literal(literal)
    }

    fn visit_plan(
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
            alloc::boxed::Box::new([
                #(#formulae),*
            ])
        }
    }

    fn visit_body_formula(&mut self, formula: &BodyFormula) -> impl ToTokens {
        match formula {
            BodyFormula::BeliefOrGoal { trigger, literal } => {
                let literal = self.visit_literal(literal);
                match trigger {
                    BodyFormulaTrigger::Add => quote! {
                        ::ember::agent::bdi::plan::Formula::Belief {
                            trigger: ::ember::agent::bdi::event::Trigger::Addition,
                            belief: #literal,
                        }
                    },
                    BodyFormulaTrigger::Remove => quote! {
                        ::ember::agent::bdi::plan::Formula::Belief {
                            trigger: ::ember::agent::bdi::event::Trigger::Deletion,
                            belief: #literal,
                        }
                    },
                    BodyFormulaTrigger::Achieve => quote! {
                        ::ember::agent::bdi::plan::Formula::Goal {
                            kind: ::ember::agent::bdi::event::GoalKind::Achieve,
                            goal: #literal,
                        }
                    },
                    BodyFormulaTrigger::Query => quote! {
                        ::ember::agent::bdi::plan::Formula::Goal {
                            kind: ::ember::agent::bdi::event::GoalKind::Query,
                            goal: #literal,
                        }
                    },
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
            LogicalExpression::Simple(SimpleLogicalExpression::Literal(literal)) => {
                let literal = self.visit_literal(literal);
                quote! { ::ember::agent::bdi::plan::QueryFormula::Literal(#literal) }
            }
            LogicalExpression::Simple(SimpleLogicalExpression::Rel(expression)) => {
                let expression = self.visit_relational_expression(expression);
                quote! { ::ember::agent::bdi::plan::QueryFormula::Relational(#expression) }
            }
            LogicalExpression::Simple(SimpleLogicalExpression::Not(expression)) => {
                let expression = self.visit_logical_expression(expression);
                quote! { ::ember::agent::bdi::plan::QueryFormula::Not(alloc::boxed::Box::new(#expression)) }
            }
            LogicalExpression::And((lhs, rhs)) => {
                let lhs = self
                    .visit_logical_expression(&LogicalExpression::Simple(lhs.clone()))
                    .into_token_stream();
                let rhs = self.visit_logical_expression(rhs);

                quote! {
                    ::ember::agent::bdi::plan::QueryFormula::Logical {
                        operator: ::ember::agent::bdi::plan::LogicalOperator::Conjunction,
                        operands: ::alloc::boxed::Box::new([#lhs, #rhs]),
                    }
                }
            }
            LogicalExpression::Or((lhs, rhs)) => {
                let lhs = self
                    .visit_logical_expression(&LogicalExpression::Simple(lhs.clone()))
                    .into_token_stream();
                let rhs = self.visit_logical_expression(rhs);

                quote! {
                    ::ember::agent::bdi::plan::QueryFormula::Logical {
                        operator: ::ember::agent::bdi::plan::LogicalOperator::Disjunction,
                        operands: alloc::boxed::Box::new([#lhs, #rhs]),
                    }
                }
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
                        operands: alloc::boxed::Box::new([#lhs, #rhs]),
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
                        operands: alloc::boxed::Box::new([#lhs, #rhs]),
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
                    ::ember::agent::bdi::term::owned::Term::Variable(
                        ::ember::agent::bdi::term::NonGround(#variable)
                    )
                }
            }
            ArithmeticFactor::Neg(factor) => {
                let factor = self.visit_arithmetic_factor(factor);
                quote! {
                    ::ember::agent::bdi::plan::ArithmeticExpression {
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
            ::ember::agent::bdi::literal::Literal::Atom {
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
            Term::Literal(Literal { negated, formula }) => {
                let formula = self.visit_atomic_formula(formula);
                quote! {
                    ::ember::agent::bdi::term::owned::Term::Literal {
                        negated: #negated,
                        structure: #formula,
                    }
                }
            }
            Term::Variable(variable) => {
                let variable = self.visit_variable(variable);
                quote! {
                    ::ember::agent::bdi::term::owned::Term::Variable(
                        ::ember::agent::bdi::term::NonGround(#variable)
                    )
                }
            }
            Term::Number(number) => quote! {
                ::ember::agent::bdi::term::owned::Term::Number(#number.into()),
            },
            Term::String(string) => {
                quote! {
                    ::ember::agent::bdi::term::owned::Term::String(#string.into())
                }
            }
        }
    }

    fn visit_variable(&mut self, Variable(name): &Variable) -> impl ToTokens {
        let var_name = self
            .variable_map
            .entry(name.clone())
            .or_insert_with(|| format_ident!("var_{}", name.to_lowercase()));
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
            Action::System(action) => {
                let action = self.visit_system_action(action);
                quote! {
                    ::ember::agent::bdi::action::Action::System(#action)
                }
            }
            Action::User(atomic_formula) => {
                let mut actual_functor = &atomic_formula.functor;
                let mut actual_args = atomic_formula.arguments.as_deref();

                if actual_functor.0 == "action" {
                    if let Some(args) = actual_args {
                        if args.len() == 1 {
                            if let Term::Literal(crate::ast::Literal { formula, .. }) = &args[0] {
                                actual_functor = &formula.functor;
                                actual_args = formula.arguments.as_deref();
                            }
                        }
                    }
                }

                let factory_ident = format_ident!("{}_action", actual_functor.0.as_str());
                let args_tokens = match actual_args {
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
                    ::ember::agent::bdi::plan::Action::User(#agent_ident::#factory_ident(#args_tokens))
                }
            }
        }
    }

    fn visit_system_action(&mut self, action: &SystemAction) -> impl ToTokens {
        match *action {}

        quote! {}
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
                ::ember::bdi::agent::plan::RelationalOperator::Compare {
                    operator: ::ember::bdi::agen::plan::CompareOperator::LessThan,
                    equal: false,
                }
            },
            RelationalOperator::Larger => quote! {
                ::ember::bdi::agent::plan::RelationalOperator::Compare {
                    operator: ::ember::bdi::agen::plan::CompareOperator::GreaterThan,
                    equal: false,
                }
            },
            RelationalOperator::SmallerEq => quote! {
                ::ember::bdi::agent::plan::RelationalOperator::Compare {
                    operator: ::ember::bdi::agen::plan::CompareOperator::LessThan,
                    equal: true,
                }
            },
            RelationalOperator::LargerEq => quote! {
                ::ember::bdi::agent::plan::RelationalOperator::Compare {
                    operator: ::ember::bdi::agen::plan::CompareOperator::GreaterThan,
                    equal: true,
                }
            },
            RelationalOperator::Equal => quote! {
                ::ember::bdi::agent::plan::RelationalOperator::Compare {
                    operator: ::ember::bdi::agen::plan::CompareOperator::EqualTo,
                    equal: true,
                }
            },
            RelationalOperator::NotEqual => quote! {
                ::ember::bdi::agent::plan::RelationalOperator::Compare {
                    operator: ::ember::bdi::agen::plan::CompareOperator::EqualTo,
                    equal: false,
                }
            },
            RelationalOperator::Unify => quote! {
                ::ember::bdi::agent::plan::RelationalOperator::Unify
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
