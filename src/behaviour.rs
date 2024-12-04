pub use self::complex::{parallel, ParallelBehaviour};
pub use self::context::Context;
pub use self::state::State;

mod complex;
mod context;
mod state;

pub trait Behaviour {
    type ParentState;

    fn action(&mut self, ctx: &mut Context, state: Self::ParentState) -> (bool, Self::ParentState);
}
