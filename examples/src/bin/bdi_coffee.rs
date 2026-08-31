#![cfg_attr(target_os = "none", no_std)]
#![cfg_attr(target_os = "none", no_main)]

use alloc::boxed::Box;
use alloc::rc::Rc;
use alloc::string::{String, ToString};
use alloc::vec;

use log::info;

use ember::Container;
use ember::agent::bdi::BdiAgent;
use ember::agent::bdi::bindings::Bindings;
use ember::agent::bdi::knowledge::base::KnowledgeBase;
use ember::agent::bdi::knowledge::belief::Knowledge;
use ember::agent::bdi::literal::{IntoLiteral, Literal};
use ember::agent::bdi::plan::action::Execute;
use ember::agent::bdi::plan::library::PlanLibrary;
use ember::agent::bdi::plan::{
    Action, BuiltinAction, Formula, GoalKind, ImpureAction, Plan, QueryFormula, Trigger,
    TriggeringEvent,
};
use ember::agent::bdi::sensor::{Percept, Perceptor};
use ember::agent::bdi::term::reference::TermRef;
use ember::agent::bdi::term::{FromTerm, FromTermError, Structure, Term};
use ember::agent::bdi::variable::Variable;

use ember_examples::setup_example;

setup_example!();

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Location(String);

impl FromTerm<'_> for Location {
    fn from_term(term: TermRef<'_>) -> Result<Self, FromTermError> {
        match term {
            TermRef::String(s) => Ok(Location(s.to_string())),
            TermRef::Literal { functor, .. } => Ok(Location(functor.0.clone())),
            _ => Err(FromTermError::InvalidType(Some("location"))),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Item(String);

impl FromTerm<'_> for Item {
    fn from_term(term: TermRef<'_>) -> Result<Self, FromTermError> {
        match term {
            TermRef::String(s) => Ok(Item(s.to_string())),
            TermRef::Literal { functor, .. } => Ok(Item(functor.0.clone())),
            _ => Err(FromTermError::InvalidType(Some("item"))),
        }
    }
}

struct Thermometer(/* Some sensor pin */);

impl Perceptor for Thermometer {
    type Percept = SensorReading;

    fn percept(&mut self) -> Option<Self::Percept> {
        Some(SensorReading { temperature: 0.0 })
    }
}

struct SensorReading {
    temperature: f32,
}

impl IntoLiteral for SensorReading {
    fn into_literal(self) -> Literal {
        Literal {
            negated: false,
            structure: Structure {
                functor: "temperature".into(),
                arguments: Some(Box::new([Term::Number(self.temperature.into())])),
            },
        }
    }
}

impl Percept for SensorReading {
    fn into_beliefs(self) -> impl IntoIterator<Item = (Trigger, Literal)> {
        [(Trigger::Addition, self.into_literal())]
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum AgentAction {
    Move { from: Variable, to: Variable },
    Buy(String),
}

impl Execute for AgentAction {
    type State = ();

    type UserAction = Self;

    fn execute<'b>(
        self,
        bindings: &Bindings<'b>,
        _context: &mut ember::agent::bdi::context::Context<Self::UserAction>,
        _knowledge: &ember::agent::bdi::knowledge::base::KnowledgeBase,
        _state: &mut Self::State,
    ) -> ember::agent::bdi::plan::action::ExecuteResult<'b, Self> {
        match self {
            AgentAction::Move { from, to } => {
                let from = bindings
                    .lookup(&from)
                    .expect("failed to lookup from in bindings");
                let to = bindings
                    .lookup(&to)
                    .expect("failed to lookup to in bindings");

                info!("[ACTION] 🏃 Moving from {from:?} to {to:?}");
            }
            AgentAction::Buy(item) => {
                info!("[ACTION] 🛒 Buying {item}");
            }
        }
        ember::agent::bdi::plan::action::ExecuteResult::Done(None)
    }
}

fn example() {
    info!("☕ Starting BDI Agent demo: The Coffee Maker ☕");
    info!("====================================================\n");

    let mut belief_base = KnowledgeBase::default();
    let beliefs = vec![
        Knowledge::from(Literal {
            negated: false,
            structure: Structure {
                functor: "at".into(),
                arguments: Some(Box::new([
                    Term::String("agent".into()),
                    Term::String("home".into()),
                ])),
            },
        }),
        Knowledge::from(Literal {
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

    info!("🧠 Agent has {belief_count} initial beliefs.");
    info!("   - at(agent, home)");
    info!("   - at(coffee_machine, kitchen)\n");

    let plan_library = define_plans();

    let initial_goal = Literal {
        negated: false,
        structure: Structure {
            functor: "make_coffee".into(),
            arguments: None,
        },
    };
    info!("🎯 Initial Goal: +!make_coffee\n");

    // 5. Create the BdiAgent instance.
    let bdi_agent = BdiAgent::<_, _, SensorReading>::new(
        "coffee-maker",
        (),
        Some(belief_base),
        plan_library,
        vec![initial_goal],
    )
    .with_sensor(Thermometer());

    // 6. Run the agent's execution cycle.
    info!("🚀 Starting agent container...\n");

    Container::new()
        .with_agent(bdi_agent)
        .start()
        .expect("container encountered an error");

    info!("\n====================================================");
    info!("☕ BDI Agent demo finished. ☕");
}

/// Creates and returns the PlanLibrary for the coffee-making agent. See the
/// `// Plan A:` .. `// Plan F:` comments below for the ASL each plan encodes.
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
            event: Literal {
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
                Some([Term::String("agent".into()), Term::Variable(v_loc.clone())]),
            ),
            QueryFormula::literal(
                false,
                "at",
                Some([Term::String("coffee_machine".into()), Term::Variable(v_loc)]),
            ),
            QueryFormula::literal(false, "have", Some([Term::String("coffee_beans".into())])),
        ])),
        body: Rc::new([
            Formula::Action(Action::Builtin(BuiltinAction::Impure(ImpureAction::Log(
                log::Level::Info,
                [Term::String(
                    "[ACTION] 💬 Enjoying a fresh cup of coffee!".into(),
                )]
                .into(),
            )))),
            Formula::Belief {
                trigger: Trigger::Addition,
                belief: Literal {
                    negated: false,
                    structure: Structure {
                        functor: "done".into(),
                        arguments: None,
                    },
                },
                silent: false,
            },
        ]),
    });

    // Plan B: Sub-goaling plan to make coffee if conditions are not met.
    // +!make_coffee <- !go_to(kitchen); !get_beans; !make_coffee.
    lib.add(Plan {
        trigger: TriggeringEvent {
            trigger: Trigger::Addition,
            goal: Some(GoalKind::Achieve),
            event: Literal {
                negated: false,
                structure: Structure {
                    functor: "make_coffee".into(),
                    arguments: None,
                },
            },
        },
        context: None, // This is the default, less specific plan.
        body: Rc::new([
            Formula::Goal {
                kind: GoalKind::Achieve,
                goal: Literal {
                    negated: false,
                    structure: Structure {
                        functor: "go_to".into(),
                        arguments: Some(Box::new([Term::String("kitchen".into())])),
                    },
                },
            },
            Formula::Goal {
                kind: GoalKind::Achieve,
                goal: Literal {
                    negated: false,
                    structure: Structure {
                        functor: "get_beans".into(),
                        arguments: Some(Box::new([])),
                    },
                },
            },
            Formula::Goal {
                kind: GoalKind::Achieve,
                goal: Literal {
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
            event: Literal {
                negated: false,
                structure: Structure {
                    functor: "go_to".into(),
                    arguments: Some(vec![Term::Variable(v_dest.clone())].into_boxed_slice()),
                },
            },
        },
        context: Some(QueryFormula::literal(
            false,
            "at",
            Some([Term::String("agent".into()), Term::Variable(v_dest.clone())]),
        )),
        body: Rc::new([Formula::Action(Action::Builtin(BuiltinAction::Impure(
            ImpureAction::Log(
                log::Level::Info,
                [
                    Term::String("[ACTION] 💬 Already at".into()),
                    Term::Variable(v_dest.clone()),
                ]
                .into(),
            ),
        )))]),
    });

    // Plan D: Move to a new location.
    // +!go_to(Dest) : at(agent, From) <- .move(From, Dest); -at(agent, From); +at(agent, Dest).
    lib.add(Plan {
        trigger: TriggeringEvent {
            trigger: Trigger::Addition,
            goal: Some(GoalKind::Achieve),
            event: Literal {
                negated: false,
                structure: Structure {
                    functor: "go_to".into(),
                    arguments: Some(vec![Term::Variable(v_dest.clone())].into_boxed_slice()),
                },
            },
        },
        context: Some(QueryFormula::literal(
            false,
            "at",
            Some([Term::String("agent".into()), Term::Variable(v_from.clone())]),
        )),
        body: Rc::new([
            Formula::Action(Action::User(AgentAction::Move {
                from: v_from.clone(),
                to: v_dest.clone(),
            })),
            Formula::Belief {
                trigger: Trigger::Deletion,
                belief: Literal {
                    negated: false,
                    structure: Structure {
                        functor: "at".into(),
                        arguments: Some(Box::new([
                            Term::String("agent".into()),
                            Term::Variable(v_from),
                        ])),
                    },
                },
                silent: false,
            },
            Formula::Belief {
                trigger: Trigger::Addition,
                belief: Literal {
                    negated: false,
                    structure: Structure {
                        functor: "at".into(),
                        arguments: Some(Box::new([
                            Term::String("agent".into()),
                            Term::Variable(v_dest.clone()),
                        ])),
                    },
                },
                silent: false,
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
            event: Literal {
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
        body: Rc::new([Formula::Action(Action::Builtin(BuiltinAction::Impure(
            ImpureAction::Log(
                log::Level::Info,
                [Term::String(
                    "[ACTION] 💬 Found coffee beans in the pantry.".into(),
                )]
                .into(),
            ),
        )))]),
    });

    // Plan F: Need to buy beans.
    // +!get_beans <- .buy(coffee_beans); +have(coffee_beans).
    lib.add(Plan {
        trigger: TriggeringEvent {
            trigger: Trigger::Addition,
            goal: Some(GoalKind::Achieve),
            event: Literal {
                negated: false,
                structure: Structure {
                    functor: "get_beans".into(),
                    arguments: None,
                },
            },
        },
        context: None,
        body: Rc::new([
            Formula::Action(Action::User(AgentAction::Buy("coffee_beans".into()))),
            Formula::Belief {
                trigger: Trigger::Addition,
                belief: Literal {
                    negated: false,
                    structure: Structure {
                        functor: "have".into(),
                        arguments: Some(Box::new([Term::String("coffee_beans".into())])),
                    },
                },
                silent: false,
            },
        ]),
    });

    lib.add(Plan {
        trigger: TriggeringEvent {
            trigger: Trigger::Addition,
            goal: None,
            event: Literal {
                negated: false,
                structure: Structure {
                    functor: "done".into(),
                    arguments: None,
                },
            },
        },
        context: None,
        body: Rc::new([Formula::Action(Action::Builtin(BuiltinAction::Impure(
            ImpureAction::StopPlatform,
        )))]),
    });

    lib
}
