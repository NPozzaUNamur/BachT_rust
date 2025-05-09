use mockall::automock;
use crate::model::{event::Event, action::Action::{Tell, Ask, Get, Nask}};
use crate::blackboard::store::StoreTrait;

#[automock]
pub trait EventHandlerTrait {
    fn new() -> Self;
    
    /// **@summary** - It handles the event and returns true if the event was handled successfully
    /// 
    /// **@param** store: &StoreTrait - The store to which applying the event's action
    /// 
    /// **@param** e: &Event - The event to handle
    /// 
    /// **@returns** - return the response to the action
    fn handle_event<S: StoreTrait + 'static>(&self, store: &S, e: &Event) -> bool;
}

pub struct EventHandler;

impl EventHandlerTrait for EventHandler {
    fn new() -> Self {
        EventHandler
    }

    fn handle_event<S: StoreTrait>(&self, store: &S, e: &Event) -> bool {
        match e {
            Event {action: Tell(token), .. } => {
                store.tell(token.clone())
            },
            Event {action: Ask(token), .. } => {
                store.ask(token)
            },
            Event {action: Nask(token), .. } => {
                store.nask(token)
            },
            Event {action: Get(token), .. } => {
                store.get(token.clone())
            }
        }
    }
}

/// ===============
/// |    TESTS    |
/// ===============

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blackboard::store::MockStoreTrait;
    use crate::model::{
        action::Action::{Tell, Ask, Get, Nask},
        event::Event
    };

    #[tokio::test]
    async fn event_handler_should_handle_tell_event() {
        let mut mock_store = MockStoreTrait::default();
        mock_store.expect_tell().times(1).returning(|_| true);
        let event = Event::new(Tell("token".into()));
        assert!(EventHandler::new().handle_event(&mock_store, &event));
    }

    #[tokio::test]
    async fn event_handler_should_handle_get_event() {
        let mut mock_store = MockStoreTrait::default();
        mock_store.expect_get().times(1).returning(|_| true);
        let event = Event::new(Get("token".into()));
        assert!(EventHandler::new().handle_event(&mock_store, &event));
    }

    #[tokio::test]
    async fn event_handler_should_handle_ask_event() {
        let mut mock_store = MockStoreTrait::default();
        mock_store.expect_ask().times(1).returning(|_| true);
        let event = Event::new(Ask("token".into()));
        assert!(EventHandler::new().handle_event(&mock_store, &event));
    }

    #[tokio::test]
    async fn event_handler_should_handle_nask_event() {
        let mut mock_store = MockStoreTrait::default();
        mock_store.expect_nask().times(1).returning(|_| true);
        let event = Event::new(Nask("token".into()));
        assert!(EventHandler::new().handle_event(&mock_store, &event));
    }
}