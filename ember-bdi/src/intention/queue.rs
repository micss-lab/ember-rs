use alloc::collections::{BTreeMap, VecDeque};

use derive_where::derive_where;

use crate::bindings::Bindings;
use crate::plan::Plan;

use super::{Intention, IntentionId};

#[derive(Debug)]
#[derive_where(Default)]
pub(crate) struct IntentionQueue<'b, A> {
    intentions: BTreeMap<IntentionId, Intention<'b, A>>,
    queue: VecDeque<IntentionId>,
    current_id: IntentionId,
}

impl<A> IntentionQueue<'_, A> {
    fn next_id(&mut self) -> IntentionId {
        let id = self.current_id;
        self.current_id += 1;
        id
    }
}

impl<'b, A: Clone> IntentionQueue<'b, A> {
    pub(crate) fn push(
        &mut self,
        plan: &'_ Plan<A>,
        bindings: Bindings<'b>,
        existing_intention: Option<IntentionId>,
    ) {
        let id = existing_intention.unwrap_or_else(|| self.next_id());
        self.intentions.entry(id).or_default().push(plan, bindings);
    }
}
