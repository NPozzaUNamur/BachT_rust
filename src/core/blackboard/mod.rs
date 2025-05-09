pub mod event_handler;
pub mod task_queue;
pub mod store;
pub mod worker;

use std::future::Future;
use std::sync::Arc;
use mockall::automock;
use task_queue::{TaskQueue, TaskQueueTrait};
use worker::{Worker, WorkerTrait};
use store::{Store, StoreTrait};
use super::model::event::Event;
use event_handler::{EventHandler, EventHandlerTrait};
use super::model::action::Action;
use super::model::task::TaskError;

#[automock]
pub trait BlackboardTrait {
    /// @summary - The constructor of the Blackboard
    /// 
    /// @returns - The Blackboard instance
    fn new() -> Self;
    
    /// @summary - Allow to interact directly with te black by sending an event
    ///
    /// @param event - The event to send to the blackboard
    ///
    /// @returns - A promise of the result of the event
    /// 
    /// @note - The synchronous version of this function is send_event_sync
    fn send_event(&self, event: Event) -> impl Future<Output = Result<bool, TaskError>>;
    
    /// @summary - Allow to interact directly with the blackboard without sending an event
    /// 
    /// @param coord_data - The coordinate data to send to the blackboard
    /// 
    /// @returns - A promise of the result of the operation
    /// 
    /// @note - The synchronous version of this function is tell_sync
    fn tell(&self, coord_data: Box<str>) -> impl Future<Output = Result<bool, TaskError>>;
    
    /// @summary - Allow to interact directly with the blackboard without sending an event
    /// 
    /// @param coord_data - The coordinate data to check the blackboard
    /// 
    /// @returns - A promise of the result of the operation
    fn ask(&self, coord_data: Box<str>) -> impl Future<Output = Result<bool, TaskError>>;
    
    /// @summary - Allow to interact directly with the blackboard without sending an event
    /// 
    /// @param coord_data - The coordinate data to get from the blackboard
    /// 
    /// @returns - A promise of the result of the operation
    fn get(&self, coord_data: Box<str>) -> impl Future<Output = Result<bool, TaskError>>;
    
    /// @summary - Allow to interact directly with the blackboard without sending an event
    /// 
    /// @param coord_data - The coordinate data to check the blackboard
    /// 
    /// @returns - A promise of the result of the operation
    fn nask(&self, coord_data: Box<str>) -> impl Future<Output = Result<bool, TaskError>>;
    
    /// @summary - Allow to clone the blackboard
    /// 
    /// @returns - A clone of the blackboard
    fn clone(&self) -> Self;
}

/// The blackboard allow interaction with the store
/// It can be cloned in order to share the access to the same share space

pub struct Blackboard<Q, W, S> 
where 
    Q: TaskQueueTrait,
    W: WorkerTrait,
    S: StoreTrait,
{
    task_queue: Q,
    worker: Arc<W>,
    store: S,
}

impl<Q, W, S> BlackboardTrait for Blackboard<Q, W, S>
where
    Q: TaskQueueTrait + Sync + Send + 'static,
    W: WorkerTrait,
    S: StoreTrait + Sync + Send + 'static,
{
    fn new() -> Self {

        let store = S::new();
        let task_queue = Q::new();
        let handler = EventHandler::new();

        Blackboard {
            task_queue: task_queue.clone(),
            worker: Arc::new(W::new(store.clone(), task_queue.clone(), handler)),
            store: store.clone(),
        }
    }


    async fn send_event(&self, event: Event) -> Result<bool, TaskError> {
        let rx = self.task_queue.add_event_to_queue(event);
        let result_channel = rx.await;
        result_channel.unwrap_or_else(|_| {
            Err(TaskError::ChannelError)
        })
    }
    
    async fn tell(&self, coord_data: Box<str>) -> Result<bool, TaskError> {
        let event = Event::new(Action::Tell(coord_data));
        self.send_event(event).await
    }
    
    async fn ask(&self, coord_data: Box<str>) -> Result<bool, TaskError> {
        let event = Event::new(Action::Ask(coord_data));
        self.send_event(event).await
    }
    
    async fn get(&self, coord_data: Box<str>) -> Result<bool, TaskError> {
        let event = Event::new(Action::Get(coord_data));
        self.send_event(event).await
    }
    
    async fn nask(&self, coord_data: Box<str>) -> Result<bool, TaskError> {
        let event = Event::new(Action::Nask(coord_data));
        self.send_event(event).await
    }
    
    fn clone(&self) -> Self {
        let store = self.store.clone();
        let task_queue = self.task_queue.clone();
        let worker = Arc::clone(&self.worker);

        Blackboard {
            task_queue,
            worker,
            store,
        }
    }
}

/// @summary - Instance a new blackboard with default concrete types
/// 
/// @returns - The blackboard instance
pub fn create_blackboard() -> Blackboard<TaskQueue, Worker, Store> {
    Blackboard::<TaskQueue, Worker, Store>::new()
}

/// ===============
/// |    TESTS    |
/// ===============

#[cfg(test)]
mod tests {
    use std::time::Duration;
    use tokio::task;
    use tokio::time::timeout;
    use super::*;
    use crate::model::action::Action;
    use store::MockStoreTrait;
    use task_queue::MockTaskQueueTrait;
    use worker::MockWorkerTrait;

    #[tokio::test]
    async fn blackboard_should_process_ok_event() {
        let mock_store = MockStoreTrait::default();
        let mut mock_task_queue = MockTaskQueueTrait::default();
        let mock_worker = Arc::new(MockWorkerTrait::default());
        
        let (tx1, rx1) = tokio::sync::oneshot::channel::<Result<bool, TaskError>>();
        let (tx2, rx2) = tokio::sync::oneshot::channel::<Result<bool, TaskError>>();
        
        mock_task_queue.expect_add_event_to_queue().times(1).return_once(move |_| {rx1});
        mock_task_queue.expect_add_event_to_queue().times(1).return_once(move |_| {rx2});
        
        let bb = Blackboard {
            task_queue: mock_task_queue,
            worker: mock_worker,
            store: mock_store,
        };
        
        let event = Event::new(Action::Tell("ok".into()));
        let pending_result = bb.send_event(event);
        
        let send_result = tx1.send(Ok(true));
        assert!(send_result.is_ok());
        
        let result = pending_result.await;
        match result {
            Ok(res) => assert!(res),
            Err(_) => panic!("Error while sending event"),
        }

        let event = Event::new(Action::Tell("ko".into()));
        let pending_result = bb.send_event(event);

        let send_result = tx2.send(Ok(false));
        assert!(send_result.is_ok());

        let result = pending_result.await;
        match result {
            Ok(res) => assert!(!res),
            Err(_) => panic!("Error while sending event"),
        }
    }
    
    #[tokio::test]
    async fn blackboard_should_handle_err_event() {
        let mock_store = MockStoreTrait::default();
        let mut mock_task_queue = MockTaskQueueTrait::default();
        let mock_worker = Arc::new(MockWorkerTrait::default());

        let (tx, rx) = tokio::sync::oneshot::channel::<Result<bool, TaskError>>();

        mock_task_queue.expect_add_event_to_queue().times(1).return_once(move |_| {rx});

        let bb = Blackboard {
            task_queue: mock_task_queue,
            worker: mock_worker,
            store: mock_store,
        };

        let event = Event::new(Action::Ask("token".into()));
        let pending_result = bb.send_event(event);

        let send_result = tx.send(Err(TaskError::UnspecifiedError));
        assert!(send_result.is_ok());

        let result = pending_result.await;
        match result {
            Ok(res) => panic!("Expected an error, but got: {}", res),
            Err(err) => assert!(matches!(err, TaskError::UnspecifiedError), "Expected an unspecified error"),
        }
    }

    #[tokio::test]
    async fn blackboard_should_handle_err_channel() {
        let mock_store = MockStoreTrait::default();
        let mut mock_task_queue = MockTaskQueueTrait::default();
        let mock_worker = Arc::new(MockWorkerTrait::default());

        let (tx, rx) = tokio::sync::oneshot::channel::<Result<bool, TaskError>>();

        mock_task_queue.expect_add_event_to_queue().times(1).return_once(move |_| {rx});

        let bb = Blackboard {
            task_queue: mock_task_queue,
            worker: mock_worker,
            store: mock_store,
        };

        let event = Event::new(Action::Ask("token".into()));
        let pending_result = bb.send_event(event);

        drop(tx); // Simulate the channel being closed

        let result = pending_result.await;
        match result {
            Ok(res) => panic!("Expected an error, but got: {}", res),
            Err(err) => assert!(matches!(err, TaskError::ChannelError), "Expected a channel error"),
        }
    }
    
    #[tokio::test]
    async fn blackboard_should_allow_direct_tell() {
        let mock_store = MockStoreTrait::default();
        let mut mock_task_queue = MockTaskQueueTrait::default();
        let mock_worker = Arc::new(MockWorkerTrait::default());

        let (tx, rx) = tokio::sync::oneshot::channel::<Result<bool, TaskError>>();

        mock_task_queue.expect_add_event_to_queue().times(1).return_once(move |_| {rx});

        let bb = Blackboard {
            task_queue: mock_task_queue,
            worker: mock_worker,
            store: mock_store,
        };
        
        let pending_result = bb.tell("token".into());

        let send_result = tx.send(Ok(true));
        assert!(send_result.is_ok());

        let result = pending_result.await;
        match result {
            Ok(res) => assert!(res),
            Err(_) => panic!("Error while sending event"),
        }
    }
    
    #[tokio::test]
    async fn blackboard_should_allow_direct_ask() {
        let mock_store = MockStoreTrait::default();
        let mut mock_task_queue = MockTaskQueueTrait::default();
        let mock_worker = Arc::new(MockWorkerTrait::default());

        let (tx, rx) = tokio::sync::oneshot::channel::<Result<bool, TaskError>>();

        mock_task_queue.expect_add_event_to_queue().times(1).return_once(move |_| {rx});

        let bb = Blackboard {
            task_queue: mock_task_queue,
            worker: mock_worker,
            store: mock_store,
        };

        let pending_result = bb.ask("token".into());

        let send_result = tx.send(Ok(true));
        assert!(send_result.is_ok());

        let result = pending_result.await;
        match result {
            Ok(res) => assert!(res),
            Err(_) => panic!("Error while sending event"),
        }
    }
    
    #[tokio::test]
    async fn blackboard_should_allow_direct_get() {
        let mock_store = MockStoreTrait::default();
        let mut mock_task_queue = MockTaskQueueTrait::default();
        let mock_worker = Arc::new(MockWorkerTrait::default());

        let (tx, rx) = tokio::sync::oneshot::channel::<Result<bool, TaskError>>();

        mock_task_queue.expect_add_event_to_queue().times(1).return_once(move |_| {rx});

        let bb = Blackboard {
            task_queue: mock_task_queue,
            worker: mock_worker,
            store: mock_store,
        };

        let pending_result = bb.get("token".into());

        let send_result = tx.send(Ok(true));
        assert!(send_result.is_ok());

        let result = pending_result.await;
        match result {
            Ok(res) => assert!(res),
            Err(_) => panic!("Error while sending event"),
        }
    }
    
    #[tokio::test]
    async fn blackboard_should_allow_direct_nask() {
        let mock_store = MockStoreTrait::default();
        let mut mock_task_queue = MockTaskQueueTrait::default();
        let mock_worker = Arc::new(MockWorkerTrait::default());

        let (tx, rx) = tokio::sync::oneshot::channel::<Result<bool, TaskError>>();

        mock_task_queue.expect_add_event_to_queue().times(1).return_once(move |_| {rx});

        let bb = Blackboard {
            task_queue: mock_task_queue,
            worker: mock_worker,
            store: mock_store,
        };

        let pending_result = bb.nask("token".into());

        let send_result = tx.send(Ok(true));
        assert!(send_result.is_ok());

        let result = pending_result.await;
        match result {
            Ok(res) => assert!(res),
            Err(_) => panic!("Error while sending event"),
        }
    }

    // Integration tests
    #[tokio::test]
    async fn blackboard_should_share_state_with_his_clone() {
        let bb = create_blackboard();
        let cloned_bb = bb.clone();
        
        let tell_event = Event::new(Action::Tell("token".into()));
        let ask_event = Event::new(Action::Ask("token".into()));
        
        let tell_result = cloned_bb.send_event(tell_event).await;
        match tell_result {
            Ok(res) => assert!(res, "Tell should always return true"),
            Err(_) => panic!("Error while sending event"),
        }
        
        let ask_result = bb.send_event(ask_event).await;
        match ask_result {
            Ok(res) => assert!(res, "Cloned blackboard and original should have the same state, then ask should return true"),
            Err(_) => panic!("Error while sending event"),
        }
    }

    #[tokio::test]
    async fn blackboard_should_be_accessible_by_multiple_threads() {
        let bb = create_blackboard();

        let cloned_bb = bb.clone();
        let task_ask = task::spawn(async move {
            loop{
                let event = Event::new(Action::Ask("token".into()));
                let result = cloned_bb.send_event(event).await;
                match result {
                    Ok(res) => {
                        if res {
                            break;
                        }
                    },
                    Err(_) => panic!("Error while sending event"),
                }
            }
        });
        
        let cloned_bb = bb.clone();
        let task_tell = task::spawn(async move {
            let event = Event::new(Action::Tell("token".into()));
            let result = cloned_bb.send_event(event).await;
        });
        
        assert!(timeout(Duration::from_secs(5), task_tell).await.is_ok());
        assert!(timeout(Duration::from_secs(5), task_ask).await.is_ok());
    }
}