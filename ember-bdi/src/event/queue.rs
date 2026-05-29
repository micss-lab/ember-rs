use alloc::collections::vec_deque::VecDeque;

use super::selector::EventSelector;
use super::{EventSource, TriggeringEvent};

/// Queue of events to be processed. Events that occured after the last handled event are
/// in FIFO order. the order of events before the last handled event is unspecified.
#[derive(Debug, Default)]
pub(crate) struct EventQueue {
    queue: VecDeque<(TriggeringEvent, EventSource)>,
}

impl EventQueue {
    pub(crate) fn push(&mut self, event: TriggeringEvent, source: EventSource) {
        self.queue.push_back((event, source));
    }

    pub(crate) fn next_event<S>(
        &mut self,
        mut selector: S,
    ) -> Option<(TriggeringEvent, EventSource)>
    where
        S: EventSelector,
    {
        let idx = self
            .queue
            .iter()
            .enumerate()
            .find_map(|(i, (e, s))| selector.should_process_event(e, s).then_some(i))?;
        Some(
            self.queue
                .swap_remove_front(idx)
                .expect("event index should exist"),
        )
    }
}
