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
        // Facts describing the status of various components.
        part(motor_1, nominal).
        part(sensor_a, nominal).
        part(valve_b, degraded).
        part(pump_c, broken).

        // ==========================================
        // RULES WITH VARIABLES IN BODY BUT NOT HEAD
        // ==========================================

        // The rule head `system_critical` has no variables (ground).
        // The rule body uses the variable `Comp`.
        // This effectively acts as an existential check: "Is there ANY Comp such that part(Comp, broken) is true?"
        system_critical :- part(Comp, broken).

        // Similarly, checking if any part is degraded.
        system_warning :- part(Comp, degraded).

        // ==========================================
        // AGENT LIFECYCLE
        // ==========================================

        // Initial goal
        !diagnose_system.

        // 1) Critical emergency plan
        +!diagnose_system : system_critical
          <- trigger_critical_alarm;
             .log("warn", "System is shutting down due to critical component failure.");
             .stop_platform().

        // 2) Warning plan
        +!diagnose_system : system_warning
          <- trigger_warning;
             .log("info", "Continuing operation with degraded components.");
             .stop_platform().

        // 3) Fallback plan (nominal)
        +!diagnose_system
          <- .log("info", "All systems nominal.");
             .stop_platform().
    }
)]
struct DiagnosticAgent;

#[bdi_actions]
impl DiagnosticAgent {
    fn trigger_critical_alarm(&mut self) {
        info!("[ACTION] 🚨 CRITICAL FAULT DETECTED IN A COMPONENT 🚨");
    }

    fn trigger_warning(&mut self) {
        info!("[ACTION] ⚠️ WARNING: COMPONENT DEGRADED ⚠️");
    }
}

fn example() {
    info!("🔍 Starting Diagnostic Demo (Variables in Rule Body) 🔍");
    info!("====================================================\n");

    Container::new()
        .with_agent(DiagnosticAgent.into_agent())
        .start()
        .expect("container encountered an error");

    info!("\n====================================================");
    info!("🔍 Diagnostic demo finished. 🔍");
}
