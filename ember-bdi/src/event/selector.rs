use super::{EventSource, TriggeringEvent};

pub trait EventSelector {
    fn should_process_event(&mut self, event: &TriggeringEvent, source: &EventSource) -> bool;
}

pub struct FirstEvent;

impl EventSelector for FirstEvent {
    fn should_process_event(&mut self, _event: &TriggeringEvent, _source: &EventSource) -> bool {
        true
    }
}
