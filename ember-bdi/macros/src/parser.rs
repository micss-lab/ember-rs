use crate::action::BuiltinAction;
use crate::ast::*;
use crate::token::FlatTokenStream;

enum BeliefOrGoal {
    Belief(Belief),
    Goal(Goal),
}

peg::parser! {
    pub grammar asl_token_stream() for FlatTokenStream {
        rule span() -> proc_macro2::Span = #{|input, pos| input.next_span(pos)}

        rule spanned<T>(r: rule<T>) -> Spanned<T>
            = span:span() node:r() {
                Spanned { node, span }
            }

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

        rule belief() -> Belief = lit:literal() { Belief(lit) }

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
            = lhs:simple_logical_expression() "&" rhs:logical_expression() { LogicalExpression::And((lhs, Box::new(rhs))) }
            / lhs:simple_logical_expression() "|" rhs:logical_expression() { LogicalExpression::Or((lhs, Box::new(rhs))) }
            / "(" expr:logical_expression() ")" { expr }
            / simple:simple_logical_expression() { LogicalExpression::Simple(simple) }

        rule simple_logical_expression() -> SimpleLogicalExpression
            = "not" expr:logical_expression() { SimpleLogicalExpression::Not(Box::new(expr)) }
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
