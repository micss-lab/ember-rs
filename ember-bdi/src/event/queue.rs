use alloc::collections::vec_deque::VecDeque;

use super::TriggeringEvent;
use super::selector::EventSelector;

/// Queue of events to be processed. Events that occured after the last handled event are
/// in FIFO order. the order of events before the last handled event is unspecified.
#[derive(Default)]
pub(crate) struct EventQueue {
    queue: VecDeque<TriggeringEvent>,
}

impl EventQueue {
    pub(crate) fn push(&mut self, event: TriggeringEvent) {
        self.queue.push_back(event);
    }

    pub(crate) fn next_event<S>(&mut self, mut selector: S) -> Option<TriggeringEvent>
    where
        S: EventSelector,
    {
        let idx = self
            .queue
            .iter()
            .enumerate()
            .find_map(|(i, e)| selector.should_process_event(e).then_some(i))?;
        Some(
            self.queue
                .swap_remove_front(idx)
                .expect("event index should exist"),
        )
    }
}
