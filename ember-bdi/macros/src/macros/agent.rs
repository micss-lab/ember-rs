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

enum Statement {
    Belief(Belief),
    Goal(Goal),
    Plan(Plan),
}

peg::parser! {
    pub grammar asl_parser() for FlatTokenStream {
        pub rule program() -> Program
            = "{" statements:statement()* "}" {
            let (mut beliefs, mut goals, mut plans) = (Vec::new(), Vec::new(), Vec::new());
            statements.into_iter().for_each(|s| {
                let span = s.span;
                match s.node {
                    Statement::Belief(belief) => beliefs.push(Spanned { node: belief, span }),
                    Statement::Goal(goal) => goals.push(Spanned { node: goal, span }),
                    Statement::Plan(plan) => plans.push(Spanned { node: plan, span }),
                }
            });

            Program {
                beliefs: beliefs.into_boxed_slice(),
                goals: goals.into_boxed_slice(),
                plans: plans.into_boxed_slice(),
            }
        }

        rule statement() -> Spanned<Statement>
            = span:span() belief:belief() "." { Spanned { node: Statement::Belief(belief), span } }
            / span:span() goal:goal() "." { Spanned { node: Statement::Goal(goal), span } }
            / span:span() plan:plan() "." { Spanned { node: Statement::Plan(plan), span } }


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
            / items:list_term() { Term::List(items) }

        rule list_term() -> Box<[Term]>
            = "[" items:term() ** "," "]" { items.into_boxed_slice() }

        rule plan() -> Plan
            = event:triggering_event() context:( ":" c:context() { c })? "<-" body:body() {
                Plan {
                    event,
                    context,
                    body,
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
            / "." action:pure_builtin_action_in_context() { SimpleLogicalExpression::Action(action) }
            / lit:literal() { SimpleLogicalExpression::Literal(lit) }
            / expr:relational_expression() { SimpleLogicalExpression::Rel(expr) }

        // Only the built-in actions with no side effects may appear in a context guard - a
        // context can be evaluated more than once per event (backtracking across candidate
        // plans, or across belief alternatives within one conjunction), so anything effectful
        // here could double-fire.
        rule pure_builtin_action() -> PureAction
            = "now" "(" variable:VARIABLE() ")" { PureAction::Now(variable) }
            / "me" "(" variable:VARIABLE() ")" { PureAction::Me(variable) }
            / "append" "(" list:list_or_variable() "," item:term() "," variable:VARIABLE() ")" { PureAction::Append(list, item, variable) }
            / "member" "(" item:term() "," list:list_or_variable() ")" { PureAction::Member(item, list) }
            / "findall" "(" template:term() "," query:logical_expression() "," variable:VARIABLE() ")" { PureAction::Findall(template, Box::new(query), variable) }
            / "min" "(" list:list_or_variable() "," minimal:term() ")" { PureAction::Min(list, minimal) }
            / "max" "(" list:list_or_variable() "," maximal:term() ")" { PureAction::Max(list, maximal) }

        rule pure_builtin_action_in_context() -> PureAction
            = pure_builtin_action()
            / expected!("a pure action usable in a context (`.now`, `.me`, `.append`, `.member`, `.findall`, `.min`, `.max`)")

        rule list_or_variable() -> ListOrVariable
            = items:list_term() { ListOrVariable::List(items) }
            / var:VARIABLE() { ListOrVariable::Variable(var) }

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
            = first:body_formula() last:(";" formula:body_formula() { formula })* {
            let mut formulae = Vec::from([first]);
            formulae.extend(last);
            Body(formulae.into_boxed_slice())
        }

        rule body_formula() -> Spanned<BodyFormula>
            = span:span() trigger:BODY_FORMULA_GOAL_TRIGGER() literal:literal() { Spanned { node: BodyFormula::Goal { trigger, literal }, span } }
            / span:span() trigger:BODY_FORMULA_BELIEF_TRIGGER() literal:literal() { Spanned { node: BodyFormula::Belief { trigger: trigger.0, literal, silent: trigger.1 }, span } }
            / span:span() "." action:builtin_action() {
                Spanned {
                    span,
                    node: BodyFormula::Action(Spanned { span, node: Action::Builtin(action) }),
                }
            }
            / span:span() lhs:relational_term() "=" rhs:relational_term() { Spanned { node: BodyFormula::Unify { lhs, rhs }, span } }
            / span:span() formula:atomic_formula() {
                Spanned {
                    span,
                    node: BodyFormula::Action(Spanned { span: formula.span, node: Action::User(formula) }),
                }
            }

        rule builtin_action() -> BuiltinAction
            = pure:pure_builtin_action() { BuiltinAction::Pure(pure) }
            / impure:impure_builtin_action() { BuiltinAction::Impure(impure) }
            / expected!("a valid system action (e.g. `.log`, `.wait`, `.at`, `.now`, `.me`, `.append`, etc.)")

        rule impure_builtin_action() -> ImpureAction
            = action_log()
            / action_stop_platform()
            / action_send()
            / action_wait()
            / action_forall()
            / action_at()

        rule action_log() -> ImpureAction
            = "log" "(" level:STRING() terms:("," t:term() { t })* ")" { ImpureAction::Log(level, terms.into_boxed_slice()) }

        rule action_stop_platform() -> ImpureAction
            = "stop_platform" ("(" ")")? { ImpureAction::StopPlatform }

        rule action_send() -> ImpureAction
            = "send" "(" aid:aid_or_variable() "," trigger:PERFORMATIVE() "," literal:literal_or_variable()
              callbacks:("," c:send_callbacks() { c })? ")" {
            ImpureAction::Send { aid, trigger, literal, callbacks: callbacks.unwrap_or_default() }
        }

        rule send_callbacks() -> Box<[(CallbackKind, Literal)]>
            = "[" items:send_callback() ** "," "]" { items.into_boxed_slice() }

        rule send_callback() -> (CallbackKind, Literal)
            = kind:callback_kind() "(" goal:literal() ")" { (kind, goal) }

        rule callback_kind() -> CallbackKind
            = "on_success" { CallbackKind::OnSuccess }
            / "on_retry" { CallbackKind::OnRetry }
            / "on_failure" { CallbackKind::OnFailure }
            / "on_complete" { CallbackKind::OnComplete }
            / expected!("a send callback kind (`on_success`, `on_retry`, `on_failure`, `on_complete`)")

        rule action_wait() -> ImpureAction
            = "wait" "(" interval_millis:MILLIS() ")" { ImpureAction::Wait { interval_millis } }

        rule action_forall() -> ImpureAction
            = "forall" "(" query:logical_expression() "," goal:literal_or_variable() ")" { ImpureAction::Forall { query, goal } }

        rule action_at() -> ImpureAction
            = "at" "(" delay_millis:MILLIS() "," goal:literal() ")" { ImpureAction::At { delay_millis, goal } }

        rule aid_or_variable() -> AidOrVariable
            = s:STRING() {?
            let (name, platform) = s.split_once('@').ok_or("an aid in the form \"name@platform\" or \"name@local\"")?;
            if name.is_empty() {
                return Err("a non-empty aid name");
            }
            let aid_platform = match platform {
                "local" => None,
                p if !p.is_empty() => Some(p.to_string()),
                _ => return Err("a non-empty aid platform, or \"local\""),
            };
            Ok(AidOrVariable::Aid { aid_name: name.to_string(), aid_platform })
        }
            / var:VARIABLE() { AidOrVariable::Variable(var) }

        rule literal_or_variable() -> LiteralOrVariable
            = l:literal() { LiteralOrVariable::Literal(l) }
            / v:VARIABLE() { LiteralOrVariable::Variable(v) }

        rule PERFORMATIVE() -> Trigger
            = s:STRING() {?
            match s.as_str() {
                "inform" => Ok(Trigger::Addition),
                "disconfirm" => Ok(Trigger::Deletion),
                _ => Err("\"inform\" or \"disconfirm\""),
            }
        }

        // TODO: Implement support for custom time units, e.g. `"5s"`.
        rule MILLIS() -> u64
            = n:NUMBER() {? (n.round() == n).then_some(n.round() as u64).ok_or("an integer number of milliseconds") }

        rule span() -> proc_macro2::Span = #{|input, pos| input.next_span(pos)}

        rule VARIABLE() -> Variable = v:TOKEN_IDENT() {?
            let v = v.to_string();
            v.starts_with(|c: char| c.is_uppercase() || c == '_')
                .then_some(Variable(v))
                .ok_or("variable")
        }

        rule ATOM() -> Atom = a:$(TOKEN_IDENT()) {?
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
            = "<=" { RelationalOperator::SmallerEq }
            / ">=" { RelationalOperator::LargerEq }
            / "==" { RelationalOperator::Equal }
            / "!=" { RelationalOperator::NotEqual }
            / "<" { RelationalOperator::Smaller }
            / ">" { RelationalOperator::Larger }
            / "=" { RelationalOperator::Unify }

        rule BODY_FORMULA_BELIEF_TRIGGER() -> (BodyFormulaBeliefTrigger, bool)
            = "+" silent:"^"? { (BodyFormulaBeliefTrigger::Add, silent.is_some()) }
            / "-" silent:"^"? { (BodyFormulaBeliefTrigger::Remove, silent.is_some()) }

        rule BODY_FORMULA_GOAL_TRIGGER() -> BodyFormulaGoalTrigger
            = "!" { BodyFormulaGoalTrigger::Achieve }
            / "?" { BodyFormulaGoalTrigger::Query }
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
            };
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
        impl #agent_ident {
            pub fn into_agent(self) -> ::ember::agent::bdi::BdiAgent<'static, #agent_ident, #agent_action, #percept_type> {
                self.into_agent_named(#agent_name)
            }

            pub fn into_agent_named(
                self,
                name: impl ::core::convert::Into<::alloc::borrow::Cow<'static, str>>
            ) -> ::ember::agent::bdi::BdiAgent<'static, #agent_ident, #agent_action, #percept_type> {
                let beliefbase = #beliefbase;
                let initial_goals = #initial_goals;
                let plan_library = #plan_library;

                ::ember::agent::bdi::BdiAgent::new(
                    name,
                    self,
                    Some(beliefbase),
                    plan_library,
                    initial_goals,
                )
            }
        }
    };

    quote! {
        #input
        #impl_
    }
}

fn generate_beliefbase(beliefs: &[Spanned<Belief>], agent_ident: &Ident) -> impl ToTokens {
    let beliefs = beliefs.iter().map(|b| {
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
            ::ember::agent::bdi::knowledge::base::KnowledgeBase::assert_no_event(&mut beliefbase, _belief);
        }
    });

    quote! { {
        let mut beliefbase = ::ember::agent::bdi::knowledge::base::KnowledgeBase::default();
        #(#beliefs)*
        beliefbase
    } }
}

fn generate_initial_goals(goals: &[Spanned<Goal>], agent_ident: &Ident) -> impl ToTokens {
    let goals = goals.iter().map(|g| {
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
    let plans = plans.iter().map(|p| {
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
