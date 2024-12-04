pub use self::complex::{parallel, ParallelBehaviour};
pub use self::context::Context;
pub use self::simple::OneShotBehaviour;
pub use self::state::State;

mod complex;
mod context;
mod simple;
mod state;

/// Implemented for types that represent a behaviour.
///
/// # Shared State
///
/// Behaviours can share state by declaring state on their parent. The behaviour is passed the
/// owned state and is required to return the (possible updated) state.
pub trait Behaviour {
    /// State passed down from the parent.
    type ParentState;

    /// Executes the behaviours action once.
    ///
    /// When the behaviour is complex (e.g., it has child behaviours), the action call is
    /// propagated to the currently scheduled child.
    fn action(&mut self, ctx: &mut Context, state: Self::ParentState) -> (bool, Self::ParentState);
}
