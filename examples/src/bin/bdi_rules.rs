#![cfg_attr(target_os = "none", no_std)]
#![cfg_attr(target_os = "none", no_main)]

use log::info;

use ember::Container;
use ember::agent::bdi::{bdi_actions, bdi_agent};

use ember_examples::setup_example;

setup_example!();

#[bdi_agent(
    asl = {
        // ==========================================
        // SYSTEM FACTS
        // ==========================================
        reactor_online.
        temp_high.
        pressure_high.
        pump_a_failed.
        backup_power_available.

        // ==========================================
        // DEEP INFERENCE RULES (GROUND)
        // ==========================================

        // Negation is handled by `not`. We use deep rule chains, disjunctions (`|`),
        // conjunctions (`&`), and negations (`not`) to stress-test the DNF conversion.
        

        pump_a_active :- not pump_a_failed.
        pump_b_active :- not pump_b_failed.

        // Disjunction and Negation
        cooling_active :- pump_a_active | pump_b_active.
        cooling_insufficient :- not cooling_active.
        cooling_insufficient :- temp_high & pump_a_failed.

        // Testing DNF and multi-level inference
        danger_level_high :- reactor_online & cooling_insufficient & pressure_high.

        reactor_meltdown_imminent :- danger_level_high & not backup_power_available.

        // Disjunction via multiple rules vs inline `|`
        // Either condition makes `needs_evacuation` true.
        needs_evacuation :- danger_level_high & not reactor_meltdown_imminent.
        needs_evacuation :- reactor_meltdown_imminent.

        initiate_scram :- danger_level_high | reactor_meltdown_imminent.

        // ==========================================
        // AGENT LIFECYCLE
        // ==========================================

        // Initial goal
        !evaluate_system_state.

        // 1) Critical emergency plan
        +!evaluate_system_state : needs_evacuation & initiate_scram
          <- trigger_alarm;
             .log("warn", "System critical. Scramming reactor and evacuating.");
             .stop_platform().

        // 2) High danger plan
        +!evaluate_system_state : danger_level_high
          <- .log("warn", "Danger high, but evacuation not yet required.");
             .stop_platform().

        // 3) Fallback plan
        +!evaluate_system_state
          <- .log("info", "System nominal.");
             .stop_platform().
    }
)]
struct NuclearReactorAgent;

#[bdi_actions]
impl NuclearReactorAgent {
    fn trigger_alarm(&mut self) {
        info!("[ACTION] 🚨 🚨 🚨 EMERGENCY ALARM TRIGGERED 🚨 🚨 🚨");
    }
}

fn example() {
    info!("☢️  Starting BDI Rules Demo (Complex Ground Rules Stress Test) ☢️");
    info!("====================================================\n");

    Container::new()
        .with_agent(NuclearReactorAgent.into_agent())
        .start()
        .expect("container encountered an error");

    info!("\n====================================================");
    info!("☢️  BDI Rules demo finished. ☢️");
}
