use super::{model::{event::Event, action::Action::{Tell, Ask, Get, Nask}}, store::StoreTrait};


pub fn handle_event(s: &dyn StoreTrait, e: Event) -> bool {
    match e.action {
        Tell(token) => {
            s.tell(token)
        },
        Ask(ref token) => {
            s.ask(token)
        },
        Nask(ref token) => {
            s.nask(token)
        },
        Get(token) => {
            s.get(token)
        }
    }
}

/// ===============
/// |    TESTS    |
/// ===============

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blackboard::{
        model::{
            action::Action::{Tell, Ask, Get, Nask},
            event::Event
        },
        store::MockStoreTrait
    };

    #[test]
    fn event_handler_should_handle_tell_event() {
        let mut mock_store = MockStoreTrait::new();
        mock_store.expect_tell().times(1).returning(|_| true);
        let event = Event {
            from: "origin".into(),
            action: Tell("token".into())
        };
        assert!(handle_event(&mock_store, event));
    }

    #[test]
    fn event_handler_should_handle_get_event() {
        let mut mock_store = MockStoreTrait::new();
        mock_store.expect_get().times(1).returning(|_| true);
        let event = Event {
            from: "origin".into(),
            action: Get("token".into())
        };
        assert!(handle_event(&mock_store, event));
    }

    #[test]
    fn event_handler_should_handle_ask_event() {
        let mut mock_store = MockStoreTrait::new();
        mock_store.expect_ask().times(1).returning(|_| true);
        let event = Event {
            from: "origin".into(),
            action: Ask("token".into())
        };
        assert!(handle_event(&mock_store, event));
    }

    #[test]
    fn event_handler_should_handle_nask_event() {
        let mut mock_store = MockStoreTrait::new();
        mock_store.expect_nask().times(1).returning(|_| true);
        let event = Event {
            from: "origin".into(),
            action: Nask("token".into())
        };
        assert!(handle_event(&mock_store, event));
    }
}