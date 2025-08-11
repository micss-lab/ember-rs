use alloc::collections::BTreeSet;

use super::BehaviourId;

#[derive(Debug, Default)]
pub struct BlockTracker {
    ids: BTreeSet<BehaviourId>,
    // TODO: Optimize this to a bitset the length of `ids`.
    blocked_ids: BTreeSet<BehaviourId>,
}

impl BlockTracker {
    pub(super) fn new(ids: impl IntoIterator<Item = BehaviourId>) -> Self {
        Self {
            ids: ids.into_iter().collect(),
            blocked_ids: BTreeSet::default(),
        }
    }

    pub(super) fn register(&mut self, id: BehaviourId) -> bool {
        self.ids.insert(id)
    }

    pub(super) fn unregister(&mut self, id: BehaviourId) -> bool {
        self.ids.remove(&id)
    }

    pub(super) fn block(&mut self, id: BehaviourId) -> bool {
        if !self.ids.contains(&id) {
            return false;
        }
        self.blocked_ids.insert(id)
    }

    pub(super) fn unblock_all(&mut self) {
        self.blocked_ids.clear();
    }

    pub(super) fn is_blocked(&self, id: BehaviourId) -> Result<bool, ()> {
        if !self.ids.contains(&id) {
            return Err(());
        }
        Ok(self.blocked_ids.contains(&id))
    }
}
