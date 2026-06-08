#![cfg_attr(target_os = "none", no_std)]
#![cfg_attr(target_os = "none", no_main)]

use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec;

use log::info;

use ember::Container;
use ember::agent::bdi::bindings::BindingLookup;
use ember::agent::bdi::knowledge::belief::Belief;
use ember::agent::bdi::knowledge::store::BeliefBase;
use ember::agent::bdi::literal::Literal;
use ember::agent::bdi::plan::action::Execute;
use ember::agent::bdi::plan::library::PlanLibrary;
use ember::agent::bdi::plan::{
    Action, BuiltinAction, Formula, GoalKind, Plan, QueryFormula, Trigger, TriggeringEvent,
};
use ember::agent::bdi::term::{NonGround, Structure, Term};
use ember::agent::bdi::variable::Variable;
use ember::agent::bdi::{BdiAgent, bdi_agent};

use ember_examples::setup_example;

setup_example!();

#[bdi_agent(asl = {
    at(agent, home).
    at(coffee_machine, kitchen).

    !make_coffee.

    +!make_coffee : at(agent, Loc) & at(coffee_machine, Loc) & have(coffee_beans)
      <- .print("Enjoying a fresh cup of coffee!");
         +done.

    +!make_coffee
      <- !go_to(kitchen);
         !get_beans;
         !make_coffee.

    +!go_to(Dest) : at(agent, Dest)
      <- .print("Already at", Dest).

    +!go_to(Dest) : at(agent, From)
      <- action(move(From, Dest));
         -at(agent, From);
         +at(agent, Dest).

    +!get_beans : have(coffee_beans)
      <- .print("Found coffee beans in the pantry.").

    +!get_beans
      <- action(buy(coffee_beans));
         +have(coffee_beans).

    +done
      <- action(stop).
})]
struct CoffeeAgent;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]

enum AgentAction {
    Move { from: Variable, to: Variable },
    Buy(String),
}

impl Execute for AgentAction {
    type State = ();

    type Action = Self;

    fn execute(
        self,
        bindings: &impl BindingLookup,
        _context: &mut ember::agent::bdi::context::Context<Self::Action>,
        _state: &mut Self::State,
    ) {
        match self {
            AgentAction::Move { from, to } => {
                let from = bindings
                    .lookup(&from)
                    .expect("failed to lookup from in bindings");
                let to = bindings
                    .lookup(&to)
                    .expect("failed to lookup to in bindings");

                info!("[ACTION] 🏃 Moving from {:?} to {:?}", from, to);
            }
            AgentAction::Buy(item) => {
                info!("[ACTION] 🛒 Buying {}", item);
            }
        }
    }
}

fn example() {
    info!("☕ Starting BDI Agent demo: The Coffee Maker ☕");
    info!("====================================================\n");

    let mut belief_base = BeliefBase::default();
    let beliefs = vec![
        Belief::from(Literal::Atom {
            negated: false,
            structure: Structure {
                functor: "at".into(),
                arguments: Some(Box::new([
                    Term::String("agent".into()),
                    Term::String("home".into()),
                ])),
            },
        }),
        Belief::from(Literal::Atom {
            negated: false,
            structure: Structure {
                functor: "at".into(),
                arguments: Some(Box::new([
                    Term::String("coffee_machine".into()),
                    Term::String("kitchen".into()),
                ])),
            },
        }),
    ];

    let belief_count = beliefs.len();
    beliefs.into_iter().for_each(|b| {
        belief_base.assert_no_event(b);
    });

    info!("🧠 Agent has {} initial beliefs.", belief_count);
    info!("   - at(agent, home)");
    info!("   - at(coffee_machine, kitchen)\n");

    let plan_library = define_plans();

    let initial_goal = {
        TriggeringEvent {
            trigger: Trigger::Addition,
            goal: Some(GoalKind::Achieve),
            event: Literal::Atom {
                negated: false,
                structure: Structure {
                    functor: "make_coffee".into(),
                    arguments: None,
                },
            },
        }
    };
    info!("🎯 Initial Goal: +!make_coffee\n");

    // 5. Create the BdiAgent instance.
    let bdi_agent = BdiAgent::<_, _, ()>::new(
        "coffee-maker",
        (),
        [],
        Some(belief_base),
        plan_library,
        vec![initial_goal],
    );

    // 6. Run the agent's execution cycle.
    info!("🚀 Starting agent container...\n");

    Container::new()
        .with_agent(bdi_agent)
        .start()
        .expect("container encountered an error");

    info!("\n====================================================");
    info!("☕ BDI Agent demo finished. ☕");
}

/// Creates and returns the PlanLibrary for the coffee-making agent.
///
/// These plans constitute the following ASL file:
///
/// ```asl
///     // AI-generated.
///
///     // **Goal: Make Coffee**
///     // This plan is chosen if the agent is in the same location as the coffee
///     // machine and has coffee beans. It is the most specific and ideal plan.
///     +!make_coffee : at(agent, Loc) & at(coffee_machine, Loc) & have(coffee_beans)
///       <- .print("Enjoying a fresh cup of coffee!").
///
///     // This is the default plan for making coffee. It's chosen if the conditions
///     // for the first plan are not met. It creates sub-goals to get the agent
///     // to the right place and to acquire the necessary ingredients. After the
///     // sub-goals are complete, it retries the original `!make_coffee` goal.
///     +!make_coffee
///       <- !go_to(kitchen);
///          !get_beans;
///          !make_coffee.
///
///
///     // **Goal: Go to a Location**
///     // This plan handles the goal of moving the agent to a new location.
///     // If the agent is already at the destination, it does nothing.
///     +!go_to(Dest) : at(agent, Dest)
///       <- .print("Already at ", Dest).
///
///     // If the agent is not at the destination, this plan is selected. It performs
///     // the `move` action and then updates its internal belief state about its location.
///     +!go_to(Dest) : at(agent, From)
///       <- .move(From, Dest);
///          -at(agent, From);
///          +at(agent, Dest).
///
///
///     // **Goal: Get Coffee Beans**
///     // This plan is for acquiring coffee beans. If the agent already has them,
///     // it simply notes that fact.
///     +!get_beans : have(coffee_beans)
///       <- .print("Found coffee beans in the pantry.").
///
///     // If the agent does not have coffee beans, this default plan is chosen.
///     // It performs the `buy` action and then adds the `have(coffee_beans)`
///     // belief to its knowledge base.
///     +!get_beans
///       <- .buy(coffee_beans);
///          +have(coffee_beans).
/// ```
fn define_plans() -> PlanLibrary<AgentAction> {
    let mut lib = PlanLibrary::default();

    // --- Plans for `!make_coffee` ---

    // Plan A: Make coffee if conditions are met.
    // +!make_coffee : at(agent, kitchen) & have(coffee_beans) <- .print("Making coffee!").
    let v_loc = Variable::new();
    lib.add(Plan {
        trigger: TriggeringEvent {
            trigger: Trigger::Addition,
            goal: Some(GoalKind::Achieve),
            event: Literal::Atom {
                negated: false,
                structure: Structure {
                    functor: "make_coffee".into(),
                    arguments: None,
                },
            },
        },
        context: Some(QueryFormula::and([
            QueryFormula::literal(
                false,
                "at",
                Some([
                    Term::String("agent".into()),
                    Term::Variable(NonGround(v_loc.clone())),
                ]),
            ),
            QueryFormula::literal(
                false,
                "at",
                Some([
                    Term::String("coffee_machine".into()),
                    Term::Variable(NonGround(v_loc)),
                ]),
            ),
            QueryFormula::literal(false, "have", Some([Term::String("coffee_beans".into())])),
        ])),
        body: Box::new([
            Formula::Action(Action::Builtin(BuiltinAction::Log(
                log::Level::Info,
                "[ACTION] 💬 Enjoying a fresh cup of coffee!".into(),
                None,
            ))),
            Formula::Belief {
                trigger: Trigger::Addition,
                belief: Literal::Atom {
                    negated: false,
                    structure: Structure {
                        functor: "done".into(),
                        arguments: None,
                    },
                },
            },
        ]),
    });

    // Plan B: Sub-goaling plan to make coffee if conditions are not met.
    // +!make_coffee <- !go_to(kitchen); !get_beans; !make_coffee.
    lib.add(Plan {
        trigger: TriggeringEvent {
            trigger: Trigger::Addition,
            goal: Some(GoalKind::Achieve),
            event: Literal::Atom {
                negated: false,
                structure: Structure {
                    functor: "make_coffee".into(),
                    arguments: None,
                },
            },
        },
        context: None, // This is the default, less specific plan.
        body: Box::new([
            Formula::Goal {
                kind: GoalKind::Achieve,
                goal: Literal::Atom {
                    negated: false,
                    structure: Structure {
                        functor: "go_to".into(),
                        arguments: Some(Box::new([Term::String("kitchen".into())])),
                    },
                },
            },
            Formula::Goal {
                kind: GoalKind::Achieve,
                goal: Literal::Atom {
                    negated: false,
                    structure: Structure {
                        functor: "get_beans".into(),
                        arguments: Some(Box::new([])),
                    },
                },
            },
            Formula::Goal {
                kind: GoalKind::Achieve,
                goal: Literal::Atom {
                    negated: false,
                    structure: Structure {
                        functor: "make_coffee".into(),
                        arguments: Some(Box::new([])),
                    },
                },
            },
        ]),
    });

    // --- Plans for `!go_to(Location)` ---
    let v_dest = Variable::new();
    let v_from = Variable::new();

    // Plan C: Already at the destination.
    // +!go_to(Dest) : at(agent, Dest) <- .print("Already at", Dest).
    lib.add(Plan {
        trigger: TriggeringEvent {
            trigger: Trigger::Addition,
            goal: Some(GoalKind::Achieve),
            event: Literal::Atom {
                negated: false,
                structure: Structure {
                    functor: "go_to".into(),
                    arguments: Some(
                        vec![Term::Variable(NonGround(v_dest.clone()))].into_boxed_slice(),
                    ),
                },
            },
        },
        context: Some(QueryFormula::literal(
            false,
            "at",
            Some([
                Term::String("agent".into()),
                Term::Variable(NonGround(v_dest.clone())),
            ]),
        )),
        body: Box::new([Formula::Action(Action::Builtin(BuiltinAction::Log(
            log::Level::Info,
            "[ACTION] 💬 Already at".into(),
            Some(Box::new([Term::Variable(NonGround(v_dest.clone()))])),
        )))]),
    });

    // Plan D: Move to a new location.
    // +!go_to(Dest) : at(agent, From) <- .move(From, Dest); -at(agent, From); +at(agent, Dest).
    lib.add(Plan {
        trigger: TriggeringEvent {
            trigger: Trigger::Addition,
            goal: Some(GoalKind::Achieve),
            event: Literal::Atom {
                negated: false,
                structure: Structure {
                    functor: "go_to".into(),
                    arguments: Some(
                        vec![Term::Variable(NonGround(v_dest.clone()))].into_boxed_slice(),
                    ),
                },
            },
        },
        context: Some(QueryFormula::literal(
            false,
            "at",
            Some([
                Term::String("agent".into()),
                Term::Variable(NonGround(v_from.clone())),
            ]),
        )),
        body: Box::new([
            Formula::Action(Action::User(AgentAction::Move {
                from: v_from.clone(),
                to: v_dest.clone(),
            })),
            Formula::Belief {
                trigger: Trigger::Deletion,
                belief: Literal::Atom {
                    negated: false,
                    structure: Structure {
                        functor: "at".into(),
                        arguments: Some(Box::new([
                            Term::String("agent".into()),
                            Term::Variable(NonGround(v_from)),
                        ])),
                    },
                },
            },
            Formula::Belief {
                trigger: Trigger::Addition,
                belief: Literal::Atom {
                    negated: false,
                    structure: Structure {
                        functor: "at".into(),
                        arguments: Some(Box::new([
                            Term::String("agent".into()),
                            Term::Variable(NonGround(v_dest.clone())),
                        ])),
                    },
                },
            },
        ]),
    });

    // --- Plans for `!get_beans` ---

    // Plan E: Already have beans.
    // +!get_beans : have(coffee_beans) <- .print("Already have coffee beans.").
    lib.add(Plan {
        trigger: TriggeringEvent {
            trigger: Trigger::Addition,
            goal: Some(GoalKind::Achieve),
            event: Literal::Atom {
                negated: false,
                structure: Structure {
                    functor: "get_beans".into(),
                    arguments: None,
                },
            },
        },
        context: Some(QueryFormula::literal(
            false,
            "have",
            Some([Term::String("coffee_beans".into())]),
        )),
        body: Box::new([Formula::Action(Action::Builtin(BuiltinAction::Log(
            log::Level::Info,
            "[ACTION] 💬 Found coffee beans in the pantry.".into(),
            None,
        )))]),
    });

    // Plan F: Need to buy beans.
    // +!get_beans <- .buy(coffee_beans); +have(coffee_beans).
    lib.add(Plan {
        trigger: TriggeringEvent {
            trigger: Trigger::Addition,
            goal: Some(GoalKind::Achieve),
            event: Literal::Atom {
                negated: false,
                structure: Structure {
                    functor: "get_beans".into(),
                    arguments: None,
                },
            },
        },
        context: None,
        body: Box::new([
            Formula::Action(Action::User(AgentAction::Buy("coffee_beans".into()))),
            Formula::Belief {
                trigger: Trigger::Addition,
                belief: Literal::Atom {
                    negated: false,
                    structure: Structure {
                        functor: "have".into(),
                        arguments: Some(Box::new([Term::String("coffee_beans".into())])),
                    },
                },
            },
        ]),
    });

    lib.add(Plan {
        trigger: TriggeringEvent {
            trigger: Trigger::Addition,
            goal: None,
            event: Literal::Atom {
                negated: false,
                structure: Structure {
                    functor: "done".into(),
                    arguments: None,
                },
            },
        },
        context: None,
        body: Box::new([Formula::Action(Action::Builtin(
            BuiltinAction::StopPlatform,
        ))]),
    });

    lib
}
