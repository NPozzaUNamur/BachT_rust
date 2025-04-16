use std::sync::Arc;
use tokio::sync::{Mutex, Notify};
use tokio::task;
use crate::blackboard::{event_handler, store, model::event::Event};
use crate::blackboard::event_queue::{worker, EventQueue, add_event_to_queue};
use crate::blackboard::socket_listener::{SocketListenerTrait, SocketListener};
use crate::blackboard::store::StoreTrait;

trait BlackboardTrait {
    /// @summary - The constructor of the Blackboard
    ///
    /// @param port - The port to listen on (u16 because port is u16 see [this wikipedia page](https://en.wikipedia.org/wiki/Registered_port), but must not be 0)
    fn new(port: u16) -> Self;

    fn new_custom(store: &dyn StoreTrait, event_queue: EventQueue, socket_listener: &dyn SocketListenerTrait) -> Self;

    /// @summary - Allow to interact directly with te black by sending an event
    ///
    /// @param event - The event to send to the blackboard
    ///
    /// @returns - A promise of the result of the event
    async fn send_event(&self, event: Event) -> bool;
}
pub(crate) struct Blackboard {

    /// The store is the main data structure of the blackboard. It is a key-value store that holds the data of the agents.
    store: dyn StoreTrait,

    event_queue: EventQueue,

    /// The socket listener is responsible for listening to incoming events from other agents.
    socket_listener: dyn SocketListenerTrait,
}

impl BlackboardTrait for Blackboard {
    fn new(port: u16) -> Self {

        let store = store::Store::new();
        let socket_listener = SocketListener::new(port);
        let event_queue = EventQueue {
            event_queue: Arc::new(Mutex::new(Vec::new())),
            notify: Arc::new(Notify::new()),
        };

        Self::new_custom(&store, event_queue, &socket_listener)
    }

    fn new_custom(store: &dyn StoreTrait, event_queue: EventQueue, socket_listener: &dyn SocketListenerTrait) -> Self {

        let blackboard = Blackboard {
            store: store,
            event_queue: event_queue,
            socket_listener: socket_listener,
        };

        // Clone for the worker
        let store_clone = store.clone();
        let event_queue_clone = event_queue.clone();

        task::spawn(async move {
            worker (
                store_clone,
                event_queue_clone,
                |store, event| {
                    event_handler::handle_event(store, event);
                },
            ).await;
        });
        blackboard
    }


    async fn send_event(&self, event: Event) -> bool {
        add_event_to_queue(&self.event_queue.clone(), event).await;
        // TODO: Communication between worker and blackboard
        true
    }
}

/// ===============
/// |    TESTS    |
/// ===============

#[cfg(test)]
mod tests {
    use tokio::time::sleep;
    use std::time::Duration;
    use super::*;
    use crate::blackboard::model::action::Action::Tell;

    #[tokio::test]
    async fn blackboard_should_process_tell_event() {

        let blackboard = Blackboard::new(8080);
        let event = Event {
            from: "origin".into(),
            action: Tell("token".into()),
        };

        blackboard.send_event(event).await;

        sleep(Duration::from_secs(5)).await;

        assert!(blackboard.store.ask("token".into()), "Token should be in the store. It can mean that the worker didn't process the event within 5 seconds");
    }

    #[tokio::test]
    async fn blackboard_should_process_ask_event() {}
}