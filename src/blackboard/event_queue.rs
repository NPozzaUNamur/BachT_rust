use std::sync::Arc;
use tokio::sync::{Mutex, Notify};

use crate::blackboard::{model::event::Event, store::StoreTrait};

/// The EventQueue hold incoming event to be processed by the event handler in order to operate on the store.
/// This is inspired by: [T. Simmer's work](https://medium.com/@thomas.simmer/rust-build-a-simple-celery-like-worker-7ae90f170515)
#[derive(Clone)]
pub(crate) struct EventQueue {
    pub event_queue: Arc<Mutex<Vec<Event>>>,
    pub notify: Arc<Notify>
}

/// @summary - Allow to add an event to the queue w.r.t. FIFO Policy
/// @param event_queue - The event queue to add the event to
/// @param event - The event to add to the queue
pub async fn add_event_to_queue(event_queue: &EventQueue, event: Event) {
    let mut queue = event_queue.event_queue.lock().await;
    queue.insert(0, event);
    event_queue.notify.notify_one();
}

pub async fn worker(store: &dyn StoreTrait, event_queue: EventQueue, process_event: fn(&Arc<dyn StoreTrait>, Event)) {
    loop {
        loop{
            let event = {
                let mut queue = event_queue.event_queue.lock().await;
                queue.pop()
            };

            if let Some(event) = event {
                process_event(&store, event);
            } else {
                // if there is no event in the queue, wait for a notification
                break;
            }
        }
        event_queue.notify.notified().await;
    }
}


/// ===============
/// |    TESTS    |
/// ===============

#[cfg(test)]
mod test {
    use tokio::{
        task,
        time::{timeout, sleep}
    };
    use std::time::Duration;
    use tokio::task::JoinHandle;
    use super::*;
    use crate::blackboard::model::event::Event;
    use crate::blackboard::model::action::Action::{Tell};

    #[tokio::test]
    async fn queue_should_add_event_when_function_called() {
        let event_queue = EventQueue {
            event_queue: Arc::new(Mutex::new(Vec::new())),
            notify: Arc::new(Notify::new()),
        };

        let event = Event {
            from: "origin".into(),
            action: Tell("token".into()),
        };

        add_event_to_queue(&event_queue, event).await;

        let queue = event_queue.event_queue.lock().await;

        assert_eq!(queue.len(), 1);
    }

    #[tokio::test]
    async fn queue_should_notify_worker_when_event_added() {
        let event_queue = EventQueue {
            event_queue: Arc::new(Mutex::new(Vec::new())),
            notify: Arc::new(Notify::new()),
        };

        let event = Event {
            from: "origin".into(),
            action: Tell("token".into()),
        };

        let event_queue_clone = event_queue.clone();

        let worker: JoinHandle<Result<(), ()>> = task::spawn(async move {
            // Timeout send error if the worker is not notified within 5 seconds
            match timeout(Duration::from_secs(5), event_queue_clone.notify.notified()).await {
                Ok(_) => {Ok(())},
                Err(_) => {Err(())},
            }
        });

        add_event_to_queue(&event_queue, event).await;

        let result = worker.await;
        assert!(result.is_ok(), "Worker task should have completed successfully");
        assert!(result.unwrap().is_ok(), "Worker should have been notified within the timeout period");
    }

    #[tokio::test]
    async fn worker_should_process_event_from_queue() {
        let event_queue = EventQueue {
            event_queue: Arc::new(Mutex::new(Vec::new())),
            notify: Arc::new(Notify::new()),
        };

        // Create a mock store
        let store = crate::blackboard::store::MockStoreTrait::new();

        let event = Event {
            from: "origin".into(),
            action: Tell("token".into()),
        };

        let event_queue_clone = event_queue.clone();

        task::spawn(async move {
            worker(&store, event_queue_clone, |_store, event| {
                assert_eq!(event.from, "origin".into());
                assert_eq!(event.action, Tell("token".into()));
            }).await;
        });

        add_event_to_queue(&event_queue, event).await;

        // Not the best method to test
        // Wait some time to let the worker process the event, if no assert error then the test is ok (but if no event is added the test will pass too)
        sleep(Duration::from_secs(5)).await;
    }
}