use alloc::collections::vec_deque::VecDeque;

use super::selector::{EventSelector, FirstEvent};
use super::{EventSource, TriggeringEvent};

/// Queue of events to be processed. Events that occured after the last handled event are
/// in FIFO order. the order of events before the last handled event is unspecified.
#[derive(Debug)]
pub(crate) struct EventQueue<Sel = FirstEvent> {
    queue: VecDeque<(TriggeringEvent, EventSource)>,
    selector: Sel,
}

impl<Sel: Default> Default for EventQueue<Sel> {
    fn default() -> Self {
        Self {
            queue: VecDeque::default(),
            selector: Sel::default(),
        }
    }
}

impl<Sel> EventQueue<Sel> {
    pub(crate) fn push(&mut self, event: TriggeringEvent, source: EventSource) {
        self.queue.push_back((event, source));
    }

    /// Configures which queued event is handled next. Replaces the default (`FirstEvent`).
    /// Changes the selector's type, so it returns a differently-typed queue.
    pub(crate) fn with_event_selector<NewSel>(self, selector: NewSel) -> EventQueue<NewSel> {
        EventQueue {
            queue: self.queue,
            selector,
        }
    }

    pub(crate) fn next_event(&mut self) -> Option<(TriggeringEvent, EventSource)>
    where
        Sel: EventSelector,
    {
        let selector = &mut self.selector;
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
