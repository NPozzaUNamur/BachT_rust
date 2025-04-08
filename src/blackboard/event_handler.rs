use super::event::Event;

pub(crate) trait EventHandlerTrait {
    /// Handle an incoming event
    fn handle_event(&self, event: &Event);
}

/// @summary - The EventHandler is responsible for handling incoming events and performing them on the store.
pub(crate) struct EventHandler {}

impl EventHandlerTrait for EventHandler {
    fn handle_event(&self, event: &Event) {
        todo!("{:?}", event)
    }
}

impl EventHandler {
}