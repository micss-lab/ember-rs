#![cfg_attr(target_os = "none", no_std)]
#![cfg_attr(target_os = "none", no_main)]

use alloc::boxed::Box;
use alloc::vec;

use ember_examples::setup_example;
use log::info;

use ember_bdi::context::Context;
use ember_bdi::knowledge::store::BeliefBase;
use ember_bdi::literal::Literal;
use ember_bdi::plan::selector::PlanSelection;
use ember_bdi::plan::store::PlanStore;
use ember_bdi::plan::{
    Action, ArithmeticExpression, CompareOperator, Formula, GoalKind, LogicalOperator, Plan,
    QueryFormula, RelationalOperator, RelationalQueryFormula, Trigger, TriggeringEvent,
};
use ember_bdi::term::{Atom, NonGround, Structure, Term};
use ember_bdi::variable::Variable;

setup_example!();

// Helpers to quickly construct terms
fn n(num: f32) -> Term {
    Term::Number(num.into())
}

fn s(str: &str) -> Term {
    Term::String(str.into())
}

fn v() -> Variable {
    Variable::new()
}

fn tv(var: &Variable) -> Term {
    Term::Variable(NonGround(var.clone()))
}

// Helpers to construct beliefs and queries
fn add_belief(bb: &mut BeliefBase, functor: &str, args: vec::Vec<Term>) {
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
    .expect("Belief must be ground (no free variables)");

    bb.assert(lit);
}

fn lit(functor: &str, args: vec::Vec<Term>) -> QueryFormula {
    QueryFormula::Literal(Literal::Atom {
        negated: false,
        structure: Structure {
            functor: Atom(functor.into()),
            arguments: if args.is_empty() {
                None
            } else {
                Some(args.into_boxed_slice())
            },
        },
    })
}

fn and(ops: vec::Vec<QueryFormula>) -> QueryFormula {
    QueryFormula::Logical {
        operator: LogicalOperator::Conjunction,
        operands: ops.into_boxed_slice(),
    }
}

fn trigger(functor: &str, args: vec::Vec<Term>, goal: Option<GoalKind>) -> TriggeringEvent {
    TriggeringEvent {
        trigger: Trigger::Addition,
        event: Literal::Atom {
            negated: false,
            structure: Structure {
                functor: Atom(functor.into()),
                arguments: if args.is_empty() {
                    None
                } else {
                    Some(args.into_boxed_slice())
                },
            },
        },
        goal,
    }
}

fn expr(t: Term) -> ArithmeticExpression {
    ArithmeticExpression::Term(t)
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

fn display_term(t: &ember_bdi::bindings::TermView) -> alloc::string::String {
    match t {
        ember_bdi::bindings::TermView::Number(n) => alloc::format!("{}", **n),
        ember_bdi::bindings::TermView::Term(Term::String(str_term)) => {
            let bytes: &[u8] = str_term.as_ref();
            alloc::format!("{}", core::str::from_utf8(bytes).unwrap_or("?"))
        }
        _ => alloc::format!("{:?}", t),
    }
}

fn example() {
    info!("Starting BDI Progress Demo...");
    info!("=============================\n");

    let mut bb = BeliefBase::default();

    info!("[STEP 1] Asserting initial beliefs to the knowledge base...");
    add_belief(&mut bb, "parent", vec![s("alice"), s("bob")]);
    add_belief(&mut bb, "parent", vec![s("bob"), s("charlie")]);
    add_belief(&mut bb, "age", vec![s("alice"), n(60.0)]);
    add_belief(&mut bb, "age", vec![s("bob"), n(35.0)]);
    add_belief(&mut bb, "age", vec![s("charlie"), n(10.0)]);

    info!("  > parent(alice, bob)");
    info!("  > parent(bob, charlie)");
    info!("  > age(alice, 60), age(bob, 35), age(charlie, 10)\n");

    let parent_var = v();
    let child_var = v();

    info!("[STEP 2] Querying: parent(X, Y)");

    info!("         Goal: Evaluate the query and dynamically bind variables X and Y.");

    let formula = lit("parent", vec![tv(&parent_var), tv(&child_var)]);

    let mut query = bb.query(&formula);

    while let Some(bindings) = query.next_bindings(None) {
        let p_str = display_term(bindings.get(&parent_var).unwrap());
        let c_str = display_term(bindings.get(&child_var).unwrap());
        info!("  -> Binding found: X = {}, Y = {}", p_str, c_str);
    }

    info!("");

    info!("[STEP 3] Querying: parent(alice, X) AND parent(X, Y)");
    info!("         Goal: Find Alice's grandchildren by chaining variable bindings.");

    let intermediate_var = v();
    let grandchild_var = v();
    let grandparent_formula = and(vec![
        lit("parent", vec![s("alice"), tv(&intermediate_var)]),
        lit("parent", vec![tv(&intermediate_var), tv(&grandchild_var)]),
    ]);

    let mut query2 = bb.query(&grandparent_formula);

    if let Some(bindings) = query2.next_bindings(None) {
        let i_str = display_term(bindings.get(&intermediate_var).unwrap());
        let g_str = display_term(bindings.get(&grandchild_var).unwrap());
        info!(
            "  -> Binding found: X = {} (intermediate), Y = {} (grandchild)",
            i_str, g_str
        );
    } else {
        info!("  -> No bindings found.");
    }

    info!("");

    info!("[STEP 4] Defining plans and evaluating them via Plan Selection...");
    let mut plan_store = PlanStore::<&'static str>::default();

    let plan_person_var = v();
    let age_var = v();

    // Plan 1: +!drive(Person) : age(Person, Age) AND Age >= 18
    let adult_plan = Plan {
        trigger: trigger("drive", vec![tv(&plan_person_var)], Some(GoalKind::Achieve)),
        context: Some(and(vec![
            lit("age", vec![tv(&plan_person_var), tv(&age_var)]),
            cmp(
                expr(tv(&age_var)),
                CompareOperator::GreaterThan,
                true, // equal = true (>=)
                expr(n(18.0)),
            ),
        ])),
        body: |_ctx| {
            Box::new([Formula::Action(Action::User(
                "Grant Driver's License and Car Keys",
            ))])
        },
    };

    let plan_person_var2 = v();
    let age_var2 = v();

    // Plan 2: +!drive(Person) : age(Person, Age) AND Age < 18
    let child_plan = Plan {
        trigger: trigger(
            "drive",
            vec![tv(&plan_person_var2)],
            Some(GoalKind::Achieve),
        ),
        context: Some(and(vec![
            lit("age", vec![tv(&plan_person_var2), tv(&age_var2)]),
            cmp(
                expr(tv(&age_var2)),
                CompareOperator::LessThan,
                false, // equal = false (<)
                expr(n(18.0)),
            ),
        ])),
        body: |_ctx| {
            Box::new([Formula::Action(Action::User(
                "Provide a Bicycle and Helmet instead",
            ))])
        },
    };

    plan_store.insert(adult_plan);
    plan_store.insert(child_plan);

    info!("  Registered 2 plans for achieving the goal `+!drive(Person)`:");
    info!("    Plan A (Adult): Context requires Person's age >= 18");
    info!("    Plan B (Child): Context requires Person's age < 18\n");

    // Test Event 1: Alice wants to drive
    info!("  Scenario 1: Alice wants to drive (Event: `+!drive(alice)`)");
    let event1 = trigger("drive", vec![s("alice")], Some(GoalKind::Achieve));
    let mut selection1 = PlanSelection::select_from_store(&event1, &plan_store);

    if let Some((plan, bindings)) = selection1.next_plan(&bb) {
        let age_val = match bindings
            .get(&age_var)
            .or_else(|| bindings.get(&age_var2))
            .unwrap()
        {
            ember_bdi::bindings::TermView::Number(n) => **n,
            _ => 0.0,
        };
        info!("    [Result] Applicable plan found and selected!");
        info!(
            "    [Details] The reasoning engine dynamically resolved Alice's age to {} via the belief base.",
            age_val
        );

        let mut ctx = Context;
        let formulas = (plan.body)(&mut ctx);
        if let Some(Formula::Action(Action::User(msg))) = formulas.first() {
            info!(
                "    [Plan Execution] Executing selected plan action: '{}'",
                msg
            );
        }
    } else {
        info!("    [Failed] No applicable plans found for Alice.");
    }
    info!("");

    // Test Event 2: Charlie wants to drive
    info!("  Scenario 2: Charlie wants to drive (Event: `+!drive(charlie)`)");
    let event2 = trigger("drive", vec![s("charlie")], Some(GoalKind::Achieve));
    let mut selection2 = PlanSelection::select_from_store(&event2, &plan_store);

    if let Some((plan, bindings)) = selection2.next_plan(&bb) {
        let age_val = match bindings
            .get(&age_var)
            .or_else(|| bindings.get(&age_var2))
            .unwrap()
        {
            ember_bdi::bindings::TermView::Number(n) => **n,
            _ => 0.0,
        };
        info!("    [Result] Applicable plan found and selected!");
        info!(
            "    [Details] The reasoning engine dynamically resolved Charlie's age to {} via the belief base.",
            age_val
        );

        let mut ctx = Context;
        let formulas = (plan.body)(&mut ctx);
        if let Some(Formula::Action(Action::User(msg))) = formulas.first() {
            info!(
                "    [Plan Execution] Executing selected plan action: '{}'",
                msg
            );
        }
    } else {
        info!("    [Failed] No applicable plans found for Charlie.");
    }

    info!("\n=============================");
    info!("BDI Demo completed successfully!");
}

