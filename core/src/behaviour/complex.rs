use super::{get_id, Behaviour, BehaviourId, Context, IntoBehaviour};

pub mod parallel;
pub mod sequential;

mod blocked;
mod macros;
pub(crate) mod scheduler;

struct ComplexBehaviour<K, Q> {
    id: BehaviourId,
    kind: K,
    queue: Q,
}
