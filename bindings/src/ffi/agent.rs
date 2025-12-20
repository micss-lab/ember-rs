use core::ffi::{CStr, c_char};

use ember::{Agent, Aid};

use super::agent_state::AgentState;
use super::behaviour::complex::{FsmBehaviour, SequentialBehaviour};
use super::behaviour::simple::{CyclicBehaviour, OneShotBehaviour, TickerBehaviour};
use super::event::Event;
use super::util::{drop_raw, from_raw, new, ref_from_raw, string_from_raw};

#[unsafe(no_mangle)]
pub extern "C" fn agent_new(
    name: *const c_char,
    agent_state: *mut AgentState,
) -> *mut Agent<'static, AgentState, Event> {
    let name = unsafe { string_from_raw(name) };
    non_null!(agent_state, "got agent state null-pointer");
    let agent_state = unsafe { from_raw(agent_state) };
    new(Agent::new(name, agent_state))
}

#[unsafe(no_mangle)]
pub extern "C" fn agent_free(agent: *mut Agent<AgentState, Event>) {
    non_null_or_bail!(agent, "attempted to free agent null-pointer");
    unsafe { drop_raw(agent) }
}

// TODO: Add more behaviours here.
#[unsafe(no_mangle)]
pub extern "C" fn agent_add_behaviour_oneshot(
    agent: *mut Agent<AgentState, Event>,
    oneshot: *mut OneShotBehaviour<Event>,
) {
    non_null!(agent, "got agent null-pointer");
    non_null!(oneshot, "got oneshot behaviour null-pointer");
    let agent = unsafe { ref_from_raw(agent) };
    let behaviour = unsafe { from_raw(oneshot) };
    agent.add_behaviour(behaviour);
}

#[unsafe(no_mangle)]
pub extern "C" fn agent_add_behaviour_cyclic(
    agent: *mut Agent<AgentState, Event>,
    cyclic: *mut CyclicBehaviour<Event>,
) {
    non_null!(agent, "got agent null-pointer");
    non_null!(cyclic, "got cyclic behaviour null-pointer");
    let agent = unsafe { ref_from_raw(agent) };
    let behaviour = unsafe { from_raw(cyclic) };
    agent.add_behaviour(behaviour);
}

#[unsafe(no_mangle)]
pub extern "C" fn agent_add_behaviour_ticker(
    agent: *mut Agent<AgentState, Event>,
    ticker: *mut TickerBehaviour<Event>,
) {
    non_null!(agent, "got agent null-pointer");
    non_null!(ticker, "got ticker behaviour null-pointer");
    let agent = unsafe { ref_from_raw(agent) };
    let behaviour = unsafe { from_raw(ticker) };
    agent.add_behaviour(behaviour);
}

#[unsafe(no_mangle)]
pub extern "C" fn agent_add_behaviour_sequential(
    agent: *mut Agent<'static, AgentState, Event>,
    sequential: *mut SequentialBehaviour<Event>,
) {
    non_null!(agent, "got agent null-pointer");
    non_null!(sequential, "got sequential behaviour null-pointer");
    let agent = unsafe { ref_from_raw(agent) };
    let behaviour = unsafe { from_raw(sequential) };
    agent.add_behaviour(behaviour);
}

#[unsafe(no_mangle)]
pub extern "C" fn agent_add_behaviour_fsm(
    agent: *mut Agent<'static, AgentState, Event>,
    fsm: *mut FsmBehaviour<Event>,
) {
    non_null!(agent, "got agent null-pointer");
    non_null!(fsm, "got fsm behaviour null-pointer");
    let agent = unsafe { ref_from_raw(agent) };
    let behaviour = unsafe { from_raw(fsm) };
    agent.add_behaviour(behaviour);
}

pub(in crate::ffi) unsafe fn aid_from_c_str_pointer(aid: *const u8) -> Aid {
    use core::str::FromStr;
    Aid::from_str(unsafe {
        CStr::from_ptr(aid)
            .to_str()
            .expect("aid string should be valid utf-8")
    })
    .expect("failed to parse string as aid")
}
