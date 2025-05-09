use std::future::Future;
use std::sync::{Arc};
use mockall::automock;
use tokio::task::JoinHandle;
use tokio::sync::Mutex;
use crate::blackboard::event_handler::EventHandlerTrait;
use crate::blackboard::store::StoreTrait;
use crate::blackboard::task_queue::TaskQueueTrait;


#[automock]
pub trait WorkerTrait {
    fn new<S, T, E>(
        store: S,
        task_queue: T,
        event_handler: E,
    ) -> Self 
    where 
        S: StoreTrait + Sync + Send + 'static,
        T: TaskQueueTrait + Sync + Send + 'static,
        E: EventHandlerTrait + Sync + Send + 'static;
    
    fn safe_stop(&self) -> impl Future<Output = ()>;
}

/// Worker manage the thread in which the job is executed
pub struct Worker {
    pub join_handler: JoinHandle<()>,
    safe_stop_signal: Arc<Mutex<bool>>, // default: false
}

impl WorkerTrait for Worker {
    fn new<S, T, E>(
        store: S,
        task_queue: T,
        event_handler: E,
    ) -> Self
    where S: StoreTrait + Sync + Send + 'static,
          T: TaskQueueTrait + Sync + Send + 'static,
          E: EventHandlerTrait + Sync + Send + 'static 
    {
        let safe_stop_signal = Arc::new(Mutex::new(false));
        let safe_stop_signal_clone = safe_stop_signal.clone();

        let join_handler = tokio::spawn(async move {
            job(store, task_queue, event_handler, safe_stop_signal_clone).await;
        });

        Worker {
            join_handler,
            safe_stop_signal,
        }
    }

    async fn safe_stop(&self) {
        *self.safe_stop_signal.lock().await = true;
    }
}

/// **@summary** - The worker's job is link to a queue, it processes the task from the queue. It is an infinite loop
/// 
/// **@param** store: impl StoreTrait + Sync - The store to which applying the event's action
/// 
/// **@param** task_queue: TaskQueue - The queue to which the worker is linked
/// 
/// **@param** event_handler: impl EventHandlerTrait - The event handler to process the events
/// 
/// **@returns** - This function live until the completion of the program
/// 
/// **@note** - This function aims to be used in a separate thread
async fn job(
    store: impl StoreTrait + Sync + 'static,
    task_queue: impl TaskQueueTrait + Sync,
    event_handler: impl EventHandlerTrait,
    safe_stop_signal: Arc<Mutex<bool>>,
) {

    // Infinite loop to process events
    loop {
        // While there is event to process in the queue
        loop {
            let task = task_queue.get_task();

            if let Some(task) = task {
                // Use ref (&) to avoid moving the event and keep the ownership
                let result = event_handler.handle_event(&store, &task.event);
                // Send the result back to the event channel
                if task.res_chanel.send(Ok(result)).is_err() {
                    // The receiver has been dropped
                    // TODO: Handle channel error
                    println!("Receiver has been dropped");
                }
            } else {
                // if there is no event in the queue, wait for a notification
                break;
            }
            if *safe_stop_signal.lock().await {
                // If the signal is set to false, stop the worker
                return;
            }
        }
        task_queue.notify().await;
        if *safe_stop_signal.lock().await {
            task_queue.cancel_notification();
            // If the signal is set to false, stop the worker
            return;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::event::Event;
    use crate::model::action::Action::{Tell, Get, Ask};
    use crate::blackboard::task_queue::MockTaskQueueTrait;
    
    use std::time::Duration;
    use std::future::pending;
    use tokio::time::{sleep, timeout};
    use crate::blackboard::event_handler::{EventHandler, MockEventHandlerTrait};
    use crate::model::task::{Task, TaskError};
    use crate::blackboard::store::MockStoreTrait;

    async fn check_result(rx: tokio::sync::oneshot::Receiver<Result<bool, TaskError>>, should_timeout: bool, should_channel_error: bool, should_worker_error: bool, should_positive_result: bool) {
        
        match timeout(Duration::from_secs(5), rx).await {
            Ok(result_channel) => {
                match result_channel {
                    Ok(result_worker) => {
                        match result_worker {
                            Ok(result) => {
                                assert_eq!(result, should_positive_result, "Worker should return a {:?}", should_positive_result);
                            },
                            Err(_) => {
                                assert!(should_worker_error, "Task should be processed successfully");
                            }
                        }
                    },
                    Err(_) => {
                        assert!(should_channel_error, "Channel should not be dropped");
                    }
                }
            },
            Err(_) => {
                assert!(should_timeout, "Timeout waiting for task to be processed");
            }
        }
    }
    
    #[tokio::test]
    async fn worker_should_process_task_from_queue() {
        
        let event = Event::new(Tell("token".into()));
        let (task, rx) = Task::new(event);
        
        let mut task_queue = MockTaskQueueTrait::default();
        // Use return_once 'cause task isn't clonable. See (https://docs.rs/mockall/latest/mockall/index.html#static-return-values)
        task_queue.expect_get_task().times(1).return_once(move || Some(task));
        task_queue.expect_get_task().times(1).returning(|| None);
        task_queue.expect_notify().times(1).returning(|| {Box::pin(pending())});

        // Create a mock store
        let mut store = MockStoreTrait::default();
        store.expect_tell().times(1).returning(|_| true);
        
        let worker = Worker::new(store, task_queue, EventHandler::new());

        check_result(rx, false, false, false, true).await;

        assert!(!worker.join_handler.is_finished(), "Worker should not be finished. Error message:\n {:?}", worker.join_handler.await.unwrap_err().to_string());
    }
    
    #[tokio::test]
    async fn worker_should_process_tell_action() {
        let event = Event::new(Tell("token".into()));
        let (task, rx) = Task::new(event);

        let mut task_queue = MockTaskQueueTrait::default();
        // Use return_once 'cause task isn't clonable. See (https://docs.rs/mockall/latest/mockall/index.html#static-return-values)
        task_queue.expect_get_task().times(1).return_once(move || Some(task));
        task_queue.expect_get_task().times(1).returning(|| None);
        task_queue.expect_notify().times(1).returning(|| {Box::pin(pending())});

        // Create a mock store
        let mut store = MockStoreTrait::default();
        store.expect_tell().times(1).returning(|_| true);

        let worker = Worker::new(store, task_queue, EventHandler::new());

        match timeout(Duration::from_secs(5), rx).await {
            Ok(result_channel) => {
                match result_channel {
                    Ok(result_worker) => {
                        match result_worker {
                            Ok(result) => {
                                assert!(result, "Worker should successfully process the task");
                            },
                            Err(_) => {
                                panic!("Task should be processed successfully");
                            }
                        }
                    },
                    Err(_) => {
                        panic!("Channel should not be dropped");
                    }
                }
            },
            Err(_) => {
                panic!("Timeout waiting for task to be processed");
            }
        }

        assert!(!worker.join_handler.is_finished(), "Worker should not be finished. Error message:\n {:?}", worker.join_handler.await.unwrap_err().to_string());
    }
    
    #[tokio::test]
    async fn worker_should_process_action_with_positive_result() {
        let event = Event::new(Get("token".into()));
        let (task, rx) = Task::new(event);

        let mut task_queue = MockTaskQueueTrait::default();
        task_queue.expect_get_task().times(1).return_once(move || Some(task));
        task_queue.expect_get_task().times(1).returning(|| None);
        task_queue.expect_notify().times(1).returning(|| {Box::pin(pending())});

        // Create a mock store
        let mut store = MockStoreTrait::default();
        store.expect_tell().times(1).returning(|_| true);
        store.expect_get().times(1).returning(|_| true);

        let worker = Worker::new(store, task_queue, EventHandler::new());

        check_result(rx, false, false, false, true).await;

        assert!(!worker.join_handler.is_finished(), "Worker should not be finished. Error message:\n {:?}", worker.join_handler.await.unwrap_err().to_string());
    }
    
    #[tokio::test]
    async fn worker_should_process_action_with_negative_result() {
        let (task, rx) = Task::new(Event::new(Ask("token".into())));
        
        let mut mock_queue = MockTaskQueueTrait::default();
        mock_queue.expect_get_task().times(1).return_once(move || Some(task));
        mock_queue.expect_get_task().times(1).returning(|| None);
        mock_queue.expect_notify().times(1).returning(|| {Box::pin(pending())});
        
        let mock_store = MockStoreTrait::default();
        
        let mut mock_handler = MockEventHandlerTrait::default();
        mock_handler.expect_handle_event().times(1).returning(|_: &MockStoreTrait, _| false);
        
        let worker = Worker::new(mock_store, mock_queue, mock_handler);
        
        check_result(rx, false, false, false, false).await;

        assert!(!worker.join_handler.is_finished(), "Worker should not be finished. Error message:\n {:?}", worker.join_handler.await.unwrap_err().to_string());
    }
    
    #[tokio::test]
    async fn worker_should_handle_empty_queue() {
        let mut mock_queue = MockTaskQueueTrait::default();
        mock_queue.expect_get_task().times(1).return_once(move || None);
        mock_queue.expect_notify().times(1).returning(|| {Box::pin(pending())});
        
        let mock_store = MockStoreTrait::default();
        
        let mut mock_handler = MockEventHandlerTrait::default();
        mock_handler.expect_handle_event::<MockStoreTrait>().times(0);
        
        let worker = Worker::new(mock_store, mock_queue, mock_handler);
        
        // await for the worker to process
        sleep(Duration::from_secs(1)).await;
        
        assert!(!worker.join_handler.is_finished(), "Worker should not be finished. Error message:\n {:?}", worker.join_handler.await.unwrap_err().to_string());
    }
    
    // TODO: Error handling when implemented in handler
    /* #[tokio::test]
    async fn worker_should_transmit_error_of_handler() {
        // 
    } */
    
    #[tokio::test]
    async fn multiple_worker_should_work_concurrently() {
        let (task1, rx1) = Task::new(Event::new(Tell("token".into())));
        let (task2, rx2) = Task::new(Event::new(Tell("token".into())));
        
        let mut mock_queue1 = MockTaskQueueTrait::default();
        let mut mock_queue2 = MockTaskQueueTrait::default();
        mock_queue1.expect_get_task().times(1).return_once(move || Some(task1));
        mock_queue1.expect_get_task().times(1).returning(|| None);
        mock_queue1.expect_notify().times(1).returning(|| {Box::pin(pending())});
        mock_queue2.expect_get_task().times(1).return_once(move || Some(task2));
        mock_queue2.expect_get_task().times(1).returning(|| None);
        mock_queue2.expect_notify().times(1).returning(|| {Box::pin(pending())});
        
        let mock_store1 = MockStoreTrait::default();
        let mock_store2 = MockStoreTrait::default();
        
        let mut mock_handler1 = MockEventHandlerTrait::default();
        let mut mock_handler2 = MockEventHandlerTrait::default();
        mock_handler1.expect_handle_event().times(1).returning(|_: &MockStoreTrait, _| true);
        mock_handler2.expect_handle_event().times(1).returning(|_: &MockStoreTrait, _| true);
        
        // 1: Begin to listen before starting thread
        let listener1 = check_result(rx1, false, false, false, true);
        let listener2 = check_result(rx2, false, false, false, true);

        let worker1 = Worker::new(mock_store1, mock_queue1, mock_handler1);

        let worker2 = Worker::new(mock_store2, mock_queue2, mock_handler2);
        
        // 2: Wait for listener to receive response from workers
        listener1.await;
        listener2.await;

        assert!(!worker1.join_handler.is_finished(), "Worker1 should not be finished. Error message:\n {:?}", worker1.join_handler.await.unwrap_err().to_string());
        assert!(!worker2.join_handler.is_finished(), "Worker1 should not be finished. Error message:\n {:?}", worker2.join_handler.await.unwrap_err().to_string());
    }
    
}