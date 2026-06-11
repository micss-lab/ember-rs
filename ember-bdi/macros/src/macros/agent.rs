use proc_macro2::{Span, TokenStream};
use quote::{ToTokens, format_ident, quote};
use syn::parse::{Parse, ParseStream};
use syn::{DeriveInput, Ident, Token, Type, TypeTuple};

use crate::BdiAgentArgs;
use crate::ast::*;
use crate::token::FlatTokenStream;

impl Parse for BdiAgentArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.is_empty() {
            return Err(input.error("expected `asl = ...`"));
        }

        let (mut asl, mut percept_type) = (None, None);

        while !input.is_empty() {
            let ident: syn::Ident = input.parse()?;
            match ident.to_string().as_str() {
                "asl" => {
                    input.parse::<Token![=]>()?;

                    if asl.is_some() {
                        return Err(syn::Error::new_spanned(
                            ident,
                            "can only use argument `asl` once",
                        ));
                    }

                    asl = Some(input.parse()?);
                }
                "percept_type" => {
                    input.parse::<Token![=]>()?;

                    if percept_type.is_some() {
                        return Err(syn::Error::new_spanned(
                            ident,
                            "can only use argument `percept_type` once",
                        ));
                    }

                    percept_type = Some(input.parse()?)
                }
                _ => {
                    return Err(syn::Error::new_spanned(
                        ident,
                        "unknown argument, expected `asl = ...`",
                    ));
                }
            }

            if !input.is_empty() {
                input.parse::<Token![,]>()?;
            }
        }

        let Some(asl) = asl else {
            return Err(input.error("expected required arguments: (`asl`)"));
        };

        Ok(BdiAgentArgs { asl, percept_type })
    }
}

enum BeliefOrGoal {
    Belief(Belief),
    Goal(Goal),
}

peg::parser! {
    pub grammar asl_parser() for FlatTokenStream {
        rule belief_or_goal() -> Spanned<BeliefOrGoal>
            = span:span() belief:belief() "." { Spanned { node: BeliefOrGoal::Belief(belief), span } }
            / span:span() goal:goal() "." { Spanned { node: BeliefOrGoal::Goal(goal), span } }

        pub rule program() -> Program
            = "{" beliefs_or_goals:belief_or_goal()* plans:plan()* "}" {
            let (beliefs, goals) = {
                let (mut beliefs, mut goals) = (Vec::new(), Vec::new());
                beliefs_or_goals.into_iter().for_each(|bg| {
                    let span = bg.span;
                    match bg.node {
                        BeliefOrGoal::Belief(belief) => beliefs.push(Spanned { node: belief, span }),
                        BeliefOrGoal::Goal(goal) => goals.push(Spanned { node: goal, span }),
                    }
                });
                (beliefs.into_boxed_slice(), goals.into_boxed_slice())
            };
            let plans = plans.into_boxed_slice();

            Program {
                beliefs,
                goals,
                plans,
            }
        }

        rule belief() -> Belief = lit:literal() belief_rule:( ":-" r:logical_expression() { r })? { Belief(lit, belief_rule) }

        rule goal() -> Goal = "!" lit:literal() { Goal(lit) }

        rule literal() -> Literal
            = neg:"~"? formula:atomic_formula() {
            Literal { negated: neg.is_some(), formula }
        }

        rule atomic_formula() -> Spanned<AtomicFormula>
            = span:span() functor:ATOM() arguments:( "(" args:term() ** "," ")" { if args.is_empty() { None } else { Some(args.into_boxed_slice()) } })? {
            Spanned {
                node: AtomicFormula {
                    functor,
                    arguments: arguments.flatten(),
                },
                span
            }
        }

        rule term() -> Term
            = lit:literal() { Term::Literal(lit) }
            / var:VARIABLE() { Term::Variable(var) }
            / num:NUMBER() { Term::Number(num) }
            / string:STRING() { Term::String(string) }

        rule plan() -> Spanned<Plan>
            = span:span() event:triggering_event() context:( ":" c:context() { c })? "<-" body:body() {
            Spanned {
                node: Plan {
                    event,
                    context,
                    body,
                },
                span
            }
        }

        rule triggering_event() -> TriggeringEvent
            = trigger:TRIGGER() goal:EVENT_GOAL()? event:literal() {
            TriggeringEvent {
                trigger,
                goal,
                event,
            }
        }

        rule context() -> Context = expr:logical_expression() { Context(expr) }

        rule logical_expression() -> LogicalExpression
            = lhs:and_expression() "|" rhs:logical_expression() { LogicalExpression::Or(Box::new((lhs, rhs))) }
            / and_expression()

        rule and_expression() -> LogicalExpression
            = lhs:simple_logical_expression() "&" rhs:and_expression() { LogicalExpression::And(Box::new((LogicalExpression::Simple(lhs), rhs))) }
            / simple:simple_logical_expression() { LogicalExpression::Simple(simple) }

        rule simple_logical_expression() -> SimpleLogicalExpression
            = "not" expr:simple_logical_expression() { SimpleLogicalExpression::Not(Box::new(expr)) }
            / "(" expr:logical_expression() ")" { SimpleLogicalExpression::Group(Box::new(expr)) }
            / lit:literal() { SimpleLogicalExpression::Literal(lit) }
            / expr:relational_expression() { SimpleLogicalExpression::Rel(expr) }

        rule relational_expression() -> RelationalExpression
            = lhs:relational_term() operator:RELATIONAL_OPERATOR() rhs:relational_term() {
            RelationalExpression {
                operator,
                operands: (lhs, rhs),
            }
        }

        rule relational_term() -> RelationalTerm
            = lit:literal() { RelationalTerm::Literal(lit) }
            / expr:arithmetic_expression() { RelationalTerm::Arithm(expr) }

        rule arithmetic_expression() -> ArithmeticExpression
            = lhs:arithmetic_term() rhs:( op:PLUS_MIN() rhs:arithmetic_expression() { (op, Box::new(rhs)) } )? {
            ArithmeticExpression {
                lhs,
                rhs,
            }
        }

        rule arithmetic_term() -> ArithmeticTerm
            = lhs:arithmetic_factor() rhs:( op:DIV_MUL() rhs:arithmetic_term() { (op, Box::new(rhs)) } )? {
            ArithmeticTerm {
                lhs,
                rhs,
            }
        }

        rule arithmetic_factor() -> ArithmeticFactor
            = num:NUMBER() { ArithmeticFactor::Number(num) }
            / var:VARIABLE() { ArithmeticFactor::Variable(var) }
            / "-" expr:arithmetic_factor() { ArithmeticFactor::Neg(Box::new(expr)) }
            / "(" expr:arithmetic_expression() ")" { ArithmeticFactor::Group(Box::new(expr)) }

        rule body() -> Body
            = first:body_formula() last:(";" formula:body_formula() { formula })* "." {
            let mut formulae = Vec::from([first]);
            formulae.extend(last);
            Body(formulae.into_boxed_slice())
        }

        rule body_formula() -> Spanned<BodyFormula>
            = span:span() trigger:BODY_FORMULA_TRIGGER() literal:literal() { Spanned { node: BodyFormula::BeliefOrGoal { trigger, literal }, span } }
            / span:span() period:"."? formula:atomic_formula() {?
                Ok(Spanned {
                    span,
                    node: BodyFormula::Action(Spanned {
                        span: formula.span,
                        node: if period.is_some() {
                            Action::Builtin(BuiltinAction::try_from(formula.node)
                                .map_err(|_| "a valid system action (e.g. `.print`, `.message`, etc.)")?)
                        } else {
                            Action::User(formula)
                        }
                    })
                })
            }

        rule span() -> proc_macro2::Span = #{|input, pos| input.next_span(pos)}

        rule VARIABLE() -> Variable = v:TOKEN_IDENT() {?
            let v = v.to_string();
            v.starts_with(|c: char| c.is_uppercase() || c == '_')
                .then_some(Variable(v))
                .ok_or("variable")
        }

        rule ATOM() -> Atom = a:$("."? TOKEN_IDENT()) {?
            let a = a.to_string();
            a.starts_with(|c: char| !(c.is_uppercase() || c == '_'))
                .then_some(Atom(a))
                .ok_or("atom")
        }

        rule NUMBER() -> f32 = l:TOKEN_LITERAL() {?
            let l = l.to_string();
            l.parse().or(Err("number"))
        }

        rule STRING() -> String = l:TOKEN_LITERAL() {?
            let l = l.to_string();
            (l.len() >= 2 && l.starts_with('"') && l.ends_with('"'))
                .then_some(l[1..(l.len() - 1)].to_string())
                .ok_or("string")
        }

        rule TRIGGER() -> Trigger
            = "+" { Trigger::Addition }
            / "-" { Trigger::Deletion }

        rule EVENT_GOAL() -> EventGoal
            = "!" { EventGoal::Achieve }
            / "?" { EventGoal::Query }

        rule TOKEN_LITERAL() -> proc_macro2::Literal = #{|input, pos| input.literal(pos)}
        rule TOKEN_IDENT() -> proc_macro2::Ident = #{|input, pos| input.ident(pos)}

        rule PLUS_MIN() -> PlusMin
            = "+" { PlusMin::Plus }
            / "-" { PlusMin::Min }

        rule DIV_MUL() -> DivMul
            = "/" { DivMul::Division }
            / "*" { DivMul::Multiplication }

        rule RELATIONAL_OPERATOR() -> RelationalOperator
            = "<" { RelationalOperator::Smaller }
            / ">" { RelationalOperator::Larger }
            / "<=" { RelationalOperator::SmallerEq }
            / ">=" { RelationalOperator::LargerEq }
            / "==" { RelationalOperator::Equal }
            / "!=" { RelationalOperator::NotEqual }
            / "=" { RelationalOperator::Unify }

        rule BODY_FORMULA_TRIGGER() -> BodyFormulaTrigger
            = "!" { BodyFormulaTrigger::Achieve }
            / "?" { BodyFormulaTrigger::Query }
            / "+" { BodyFormulaTrigger::Add }
            / "-" { BodyFormulaTrigger::Remove }
    }
}

pub(crate) fn expand(args: BdiAgentArgs, input: DeriveInput) -> TokenStream {
    use heck::ToKebabCase;

    let Program {
        beliefs,
        goals,
        plans,
    } = match asl_parser::program(&FlatTokenStream::new(args.asl)) {
        Ok(p) => p,
        Err(err) => {
            let msg = format!("expected {}", err.expected);
            let span = err.location.0;
            let compile_err = syn::Error::new(span, msg).to_compile_error();
            return quote! {
                #input
                #compile_err
            }
            .into();
        }
    };

    let agent_ident = &input.ident;
    let percept_type = args.percept_type.unwrap_or_else(|| {
        // Unit type.
        Type::Tuple(TypeTuple {
            paren_token: syn::token::Paren(Span::call_site()),
            elems: syn::punctuated::Punctuated::new(),
        })
    });

    let beliefbase = generate_beliefbase(&beliefs, agent_ident);
    let initial_goals = generate_initial_goals(&goals, agent_ident);
    let plan_library = generate_plan_library(&plans, agent_ident);

    let agent_name = agent_ident.to_string().to_kebab_case();
    let agent_action = format_ident!("{}Action", agent_ident);

    let impl_ = quote! {
        impl From<#agent_ident> for ::ember::agent::bdi::BdiAgent<'static, #agent_ident, #agent_action, #percept_type> {
            fn from(agent: #agent_ident) -> Self {
                let beliefbase = #beliefbase;
                let initial_goals = #initial_goals;
                let plan_library = #plan_library;

                ::ember::agent::bdi::BdiAgent::new(
                    #agent_name,
                    agent,
                    Some(beliefbase),
                    plan_library,
                    initial_goals,
                )
            }
        }

        impl #agent_ident {
            fn into_agent(self) -> ::ember::agent::bdi::BdiAgent<'static, #agent_ident, #agent_action, #percept_type> {
                self.into()
            }
        }
    };

    quote! {
        #input
        #impl_
    }
}

fn generate_beliefbase(beliefs: &[Spanned<Belief>], agent_ident: &Ident) -> impl ToTokens {
    let beliefs = beliefs.into_iter().map(|b| {
        let span = b.span;
        let mut visitor = AstVisitor::new(agent_ident.clone());
        let belief = visitor.visit_belief(&b.node).into_token_stream();
        let variables = visitor
            .variable_map
            .into_values()
            .map(|v| {
                quote! {
                    let #v = ::ember::agent::bdi::variable::Variable::new();
                }
            })
            .collect::<Vec<_>>();

        quote::quote_spanned! { span=>
            let _belief = {
                #(#variables)*
                #belief
            };
            ::ember::agent::bdi::knowledge::store::BeliefBase::assert_no_event(&mut beliefbase, _belief);
        }
    });

    quote! { {
        let mut beliefbase = ::ember::agent::bdi::knowledge::store::BeliefBase::default();
        #(#beliefs)*
        beliefbase
    } }
}

fn generate_initial_goals(goals: &[Spanned<Goal>], agent_ident: &Ident) -> impl ToTokens {
    let goals = goals.into_iter().map(|g| {
        let span = g.span;
        let mut visitor = AstVisitor::new(agent_ident.clone());
        let goal = visitor.visit_goal(&g.node).into_token_stream();
        let variables = visitor
            .variable_map
            .into_values()
            .map(|v| {
                quote! {
                    let #v = ::ember::agent::bdi::variable::Variable::new();
                }
            })
            .collect::<Vec<_>>();

        quote::quote_spanned! { span=>
            let _goal = {
                #(#variables)*
                #goal
            };
            goals.push(_goal);
        }
    });

    quote! { {
        let mut goals = ::alloc::vec::Vec::new();
        #(#goals)*
        goals
    } }
}

fn generate_plan_library(plans: &[Spanned<Plan>], agent_ident: &Ident) -> impl ToTokens {
    let plans = plans.into_iter().map(|p| {
        let span = p.span;
        let mut visitor = AstVisitor::new(agent_ident.clone());
        let plan = visitor.visit_plan(&p.node).into_token_stream();
        let variables = visitor.variable_map.into_values().map(|v| {
            quote! {
                let #v = ::ember::agent::bdi::variable::Variable::new();
            }
        });

        quote::quote_spanned! { span=>
            let _plan = {
                #(#variables)*
                #plan
            };
            plans.add(_plan);
        }
    });

    quote! { {
        let mut plans = ::ember::agent::bdi::plan::library::PlanLibrary::default();
        #(#plans)*
        plans
    } }
}
