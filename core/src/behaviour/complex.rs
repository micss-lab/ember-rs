use super::{get_id, Behaviour, BehaviourId, Context, IntoBehaviour};

pub mod parallel;
pub mod sequential;

mod macros;
pub(crate) mod queue;

struct ComplexBehaviour<K, Q> {
    id: BehaviourId,
    kind: K,
    queue: Q,
}
