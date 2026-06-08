#![cfg_attr(target_os = "none", no_std)]
#![cfg_attr(target_os = "none", no_main)]

use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec;

use ember::agent::bdi::context::Context;
use log::info;

use ember::Container;
use ember::agent::bdi::bindings::BindingLookup;
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

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct TermOrVariable;

impl Resolve for TermOrVariable {
    todo!()
}

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

/// ```rust
///     #[bdi_actions]
///     impl CoffeeAgent {
///         fn move_location(&mut self, _context: &mut Context<CoffeeAgentAction>, from: Location, to: Location) {
///
///             info!("[ACTION] 🏃 Moving from {:?} to {:?}", from, to);
///         }
///
///         fn buy(&mut self, _context: &mut Context<CoffeeAgentAction>, item: Item) {
///             info!("[ACTION] 🛒 Buying {}", item);
///         }
///     }
/// ```
// 1. The original impl block is emitted, plus your action factories
impl CoffeeAgent {
    fn move_location(&mut self, context: &mut Context<CoffeeAgentAction>, from: Location, to: Location) {
        info!("[ACTION] 🏃 Moving from {:?} to {:?}", from, to);
    }

    // Generated factory: makes it easy for the ASL parser to instantiate actions
    // without knowing the enum structure.
    pub fn move_location_action(from: TermOrVariable, to: TermOrVariable) -> CoffeeAgentAction {
        CoffeeAgentAction::MoveLocation { from, to }
    }

	// fn buy(&mut self, ...) { ... }

	// pub fn buy_action(...) -> CoffeeAgentAction { ... }
}

// 2. The generated Enum
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum CoffeeAgentAction {
    MoveLocation { from: TermOrVariable, to: TermOrVariable },
    Buy { item: TermOrVariable },
}

// 3. The generated Execute impl
impl Execute for CoffeeAgentAction {
    type State = CoffeeAgent;
    type Action = Self;

    fn execute(
        self,
        bindings: &impl BindingLookup,
        context: &mut Context<Self::Action>,
        state: &mut Self::State,
    ) {
        match self {
            CoffeeAgentAction::MoveLocation { from, to } => {
                // Step 1: Resolve BDI variables
                let from_resolved = from.resolve(bindings); 
                let to_resolved = to.resolve(bindings);

                // Step 2: User's custom FromTerm trait 
                // (You'll need to decide how to handle conversion failures here)
                let from_typed = Location::from_term(from_resolved);
                let to_typed = Location::from_term(to_resolved);

                // Step 3: Invoke the user's original method!
                state.move_location(context, from_typed, to_typed);
            }
            CoffeeAgentAction::Buy { item } => {
                let item_resolved = item.resolve(bindings);
                let item_typed = Item::from_term(item_resolved);
                
                state.buy(context, item_typed);
            }
        }
    }
}

fn example() {
    info!("☕ Starting BDI Agent demo: The Coffee Maker ☕");
    info!("====================================================\n");

    info!("🧠 Agent has 2 initial beliefs.");
    info!("   - at(agent, home)");
    info!("   - at(coffee_machine, kitchen)\n");

    info!("🎯 Initial Goal: !make_coffee\n");

    // 6. Run the agent's execution cycle.
    info!("🚀 Starting agent container...\n");

    Container::new()
        .with_agent(BdiAgent::from(CoffeeAgent))
        .start()
        .expect("container encountered an error");

    info!("\n====================================================");
    info!("☕ BDI Agent demo finished. ☕");
}
