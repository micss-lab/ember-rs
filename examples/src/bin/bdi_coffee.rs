#![cfg_attr(target_os = "none", no_std)]
#![cfg_attr(target_os = "none", no_main)]

use alloc::boxed::Box;
use alloc::vec;

use log::{debug, info};

use ember_core::agent::Agent as EmberAgent;
use ember_core::context::ContainerContext;

use ember_bdi::agent::{Agent, BdiAgent};
use ember_bdi::knowledge::belief::Belief;
use ember_bdi::knowledge::store::BeliefBase;
use ember_bdi::literal::Literal;
use ember_bdi::plan::library::PlanLibrary;
use ember_bdi::plan::{Action, Formula, GoalKind, Plan, QueryFormula, Trigger, TriggeringEvent};
use ember_bdi::term::{NonGround, Structure, Term};
use ember_bdi::variable::Variable;

use ember_examples::setup_example;

setup_example!();

#[derive(Debug)]
struct CoffeeAgent;

impl Agent for CoffeeAgent {
    type Action = Structure;
    type Percept = ();

    /// This function is called by the BDI engine when a plan executes a custom action.
    fn perform_action(
        &mut self,
        action: Structure,
        _context: &mut ember_bdi::context::Context<Structure>,
    ) {
        match action.functor.0.as_ref() {
            "print" => {
                let arg = action
                    .arguments
                    .and_then(|args| args.get(0).cloned())
                    .unwrap_or(Term::String("".into()));
                info!("[ACTION] 💬 {:?}", arg);
            }
            "move" => {
                let from = action
                    .arguments
                    .as_ref()
                    .and_then(|args| args.get(0).cloned())
                    .unwrap_or(Term::String("??".into()));
                let to = action
                    .arguments
                    .and_then(|args| args.get(1).cloned())
                    .unwrap_or(Term::String("??".into()));
                info!("[ACTION] 🏃 Moving from {:?} to {:?}", from, to);
            }
            "buy" => {
                let item = action
                    .arguments
                    .and_then(|args| args.get(0).cloned())
                    .unwrap_or(Term::String("nothing".into()));
                info!("[ACTION] 🛒 Buying {:?}", item);
            }
            _ => {
                info!("[ACTION] ❓ Unknown action: {:?}", action);
            }
        }
    }

    fn handle_percept(
        &mut self,
        _percept: Self::Percept,
        _knowledge: &mut ember_bdi::knowledge::store::BeliefBase,
    ) {
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
        belief_base.assert(b);
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
    let mut bdi_agent = BdiAgent::new(
        "coffee-maker",
        CoffeeAgent,
        [],
        Some(belief_base),
        plan_library,
        vec![initial_goal],
    );

    // 6. Run the agent's execution cycle.
    info!("🚀 Starting agent execution loop...\n");
    let mut container_context = ContainerContext::default();
    let mut step = 0;
    const MAX_STEPS: u32 = 15;

    while step < MAX_STEPS {
        info!("--- Agent update: Step {} ---", step + 1);
        if bdi_agent.update(&mut container_context) {
            info!("\n✅ Agent has completed its tasks.");
            break;
        }
        step += 1;
        if step == MAX_STEPS {
            debug!("{bdi_agent:#?}");
            info!("\n⚠️ Agent reached max steps without completing tasks.");
        }
    }

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
fn define_plans() -> PlanLibrary<Structure> {
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
        body: Box::new([Formula::Action(Action::User(Structure {
            functor: "print".into(),
            arguments: Some(Box::new([Term::String(
                "Enjoying a fresh cup of coffee!".into(),
            )])),
        }))]),
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
        body: Box::new([Formula::Action(Action::User(Structure {
            functor: "print".into(),
            arguments: Some(Box::new([
                Term::String("Already at ".into()),
                Term::Variable(NonGround(v_dest.clone())),
            ])),
        }))]),
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
            Formula::Action(Action::User(Structure {
                functor: "move".into(),
                arguments: Some(Box::new([
                    Term::Variable(NonGround(v_from.clone())),
                    Term::Variable(NonGround(v_dest.clone())),
                ])),
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
        body: Box::new([Formula::Action(Action::User(Structure {
            functor: "print".into(),
            arguments: Some(Box::new([Term::String(
                "Found coffee beans in the pantry.".into(),
            )])),
        }))]),
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
            Formula::Action(Action::User(Structure {
                functor: "buy".into(),
                arguments: Some(Box::new([Term::String("coffee_beans".into())])),
            })),
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

    lib
}
