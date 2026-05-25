use super::TriggeringEvent;

pub trait EventSelector {
    fn should_process_event(&mut self, event: &TriggeringEvent) -> bool;
}

pub struct FirstEvent;

impl EventSelector for FirstEvent {
    fn should_process_event(&mut self, _event: &TriggeringEvent) -> bool {
        true
    }
}
