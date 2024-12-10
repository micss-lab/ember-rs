pub use self::cyclic::CyclicBehaviour;
pub use self::oneshot::OneShotBehaviour;
pub use self::ticker::TickerBehaviour;

use super::{Behaviour, Context, State};

mod cyclic;
mod oneshot;
mod ticker;

/// State stored inside a simple behaviour.
pub trait SimpleBehaviourState {
    /// Whether the behaviour has finished and should be removed from the
    /// behaviour pool.
    fn finished(&self) -> bool;
}

impl SimpleBehaviourState for () {
    fn finished(&self) -> bool {
        false
    }
}
